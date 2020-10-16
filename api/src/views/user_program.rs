use std::convert::TryInto;

use crate::{
    error::{DbErrorConverter, ResponseResult},
    models,
    schema::{
        hardware_specs, program_specs, user_program_records, user_programs,
    },
    views::{RequestContext, View},
};
use diesel::{
    ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, Table,
};
use gdlk::{
    error::{CompileError, RuntimeError, WithSource},
    Compiler, Machine,
};
use uuid::Uuid;
use validator::Validate;

/// Create a new user_program
pub struct CreateUserProgramView<'a> {
    pub context: &'a RequestContext,
    pub program_spec_id: Uuid,
    pub file_name: &'a str,
    pub source_code: &'a str,
}

impl<'a> View for CreateUserProgramView<'a> {
    type Output = models::UserProgram;

    fn check_permissions(&self) -> ResponseResult<()> {
        Ok(())
    }

    fn execute_internal(&self) -> ResponseResult<Self::Output> {
        let user = self.context.user()?;
        let new_user_program = models::NewUserProgram {
            user_id: user.id,
            program_spec_id: self.program_spec_id,
            record_id: None,
            file_name: self.file_name,
            source_code: self.source_code,
        };
        new_user_program.validate()?;

        // Insert the new row and return the whole row
        let result: Result<models::UserProgram, _> = new_user_program
            .insert()
            .returning(user_programs::table::all_columns())
            .get_result(self.context.db_conn());

        DbErrorConverter {
            // Given program spec ID was invalid. Note: this can also indicate
            // an invalid user ID which would be a server-side bug, but fuck it
            fk_violation_to_not_found: true,
            // UserProgram already exists with this program spec/file name
            unique_violation_to_exists: true,
            ..Default::default()
        }
        .convert(result)
    }
}

/// Update an existing user_program
pub struct UpdateUserProgramView<'a> {
    pub context: &'a RequestContext,
    pub id: Uuid,
    pub file_name: Option<&'a str>,
    pub source_code: Option<&'a str>,
}

impl<'a> View for UpdateUserProgramView<'a> {
    type Output = Option<models::UserProgram>;

    fn check_permissions(&self) -> ResponseResult<()> {
        Ok(())
    }

    fn execute_internal(&self) -> ResponseResult<Self::Output> {
        let user = self.context.user()?;
        let modified_user_program = models::ModifiedUserProgram {
            id: self.id,
            record_id: match self.source_code {
                None => None,
                Some(_) => Some(None),
            },
            file_name: self.file_name,
            source_code: self.source_code,
        };
        modified_user_program.validate()?;

        // Update the row, returning the new value. If the row doesn't
        // exist, this will return None.
        let result: Result<Option<models::UserProgram>, _> =
            diesel::update(models::UserProgram::find_for_user(
                modified_user_program.id,
                user.id,
            ))
            .set(modified_user_program)
            .returning(user_programs::table::all_columns())
            .get_result(self.context.db_conn())
            .optional();

        DbErrorConverter {
            // UserProgram already exists with this program spec/file name
            unique_violation_to_exists: true,
            // No update fields were given
            query_builder_to_no_update: true,
            ..Default::default()
        }
        .convert(result)
    }
}

/// Duplicate an existing user_program
pub struct CopyUserProgramView<'a> {
    pub context: &'a RequestContext,
    pub id: Uuid,
}

impl<'a> View for CopyUserProgramView<'a> {
    type Output = Option<models::UserProgram>;

    fn check_permissions(&self) -> ResponseResult<()> {
        Ok(())
    }

    fn execute_internal(&self) -> ResponseResult<Self::Output> {
        let conn = self.context.db_conn();
        let user = self.context.user()?;

        // The user has to own the user program to copy it
        let existing_user_program: Option<models::UserProgram> =
            models::UserProgram::find_for_user(self.id, user.id)
                .get_result(conn)
                .optional()?;

        // If the requested user_program exists (for the given user), copy it
        Ok(match existing_user_program {
            None => None,
            Some(user_program) => {
                Some(
                    models::NewUserProgram {
                        user_id: user.id,
                        program_spec_id: user_program.program_spec_id,
                        record_id: None,
                        // Generate a new file name
                        file_name: &format!("{} copy", &user_program.file_name),
                        source_code: &user_program.source_code,
                    }
                    .insert()
                    .returning(user_programs::all_columns)
                    .get_result(conn)?,
                )
            }
        })
    }
}

/// Delete an existing user_program
pub struct DeleteUserProgramView<'a> {
    pub context: &'a RequestContext,
    pub id: Uuid,
}

impl<'a> View for DeleteUserProgramView<'a> {
    type Output = Option<Uuid>;

    fn check_permissions(&self) -> ResponseResult<()> {
        Ok(())
    }

    fn execute_internal(&self) -> ResponseResult<Self::Output> {
        let user = self.context.user()?;

        // User has to own the program to delete it
        Ok(
            diesel::delete(models::UserProgram::find_for_user(
                self.id, user.id,
            ))
            .returning(user_programs::columns::id)
            .get_result(self.context.db_conn())
            .optional()?,
        )
    }
}

/// Execute a user_program and, if successful, store its result in the DB
pub struct ExecuteUserProgramView<'a> {
    pub context: &'a RequestContext,
    /// ID of the user_program to execute
    pub id: Uuid,
}

/// The possible outcomes of a user_program execution.
// The `Machine` variants are a lot larger, but in practice `Accepted` should be
// be used WAY more than the others (because the UI will only run this request
// after a successful execution). So it's not worth boxing the machine.
#[allow(clippy::large_enum_variant)]
pub enum ExecuteUserProgramOutput {
    /// Something went wrong during compilation
    CompileError(WithSource<CompileError>),
    /// Something went wrong during execution
    RuntimeError(WithSource<RuntimeError>),
    /// Program terminated normally, but the final state didn't match the
    /// expectation (which is defined by the program spec)
    Rejected { machine: Machine },
    /// Program terminated with expected state
    Accepted {
        machine: Machine,
        record: models::UserProgramRecord,
    },
}

impl<'a> ExecuteUserProgramView<'a> {
    /// Save a `user_program_records` row for this execution. This inserts a
    /// new row into that table, then updates our row in `user_programs` to
    /// point to that record as its latest execution.
    fn save_record(
        &self,
        program_spec_id: Uuid,
        machine: &Machine,
    ) -> ResponseResult<models::UserProgramRecord> {
        let conn = self.context.db_conn();
        let user = self.context.user()?;
        let program = machine.program();

        // Program was successful, store performance stats in the DB
        let record: models::UserProgramRecord = models::NewUserProgramRecord {
            user_id: user.id,
            program_spec_id,
            source_code: &machine.source_code(),
            // If any of these go out of range for i32, other shit will have
            // broken already, so these conversions are "safe"
            cpu_cycles: machine.cycle_count().try_into().unwrap(),
            instructions: program.num_instructions().try_into().unwrap(),
            registers_used: program
                .num_user_registers_referenced()
                .try_into()
                .unwrap(),
            stacks_used: program.num_stacks_referenced().try_into().unwrap(),
        }
        .insert()
        .returning(user_program_records::all_columns)
        .get_result(conn)?;

        // The user_program should always point to the record of the last run
        // Update it accordingly
        diesel::update(user_programs::table.find(self.id))
            .set(user_programs::columns::record_id.eq(record.id))
            .execute(conn)?;

        Ok(record)
    }
}

impl<'a> View for ExecuteUserProgramView<'a> {
    type Output = Option<ExecuteUserProgramOutput>;

    fn check_permissions(&self) -> ResponseResult<()> {
        Ok(())
    }

    fn execute_internal(&self) -> ResponseResult<Self::Output> {
        let user = self.context.user()?;
        let conn = self.context.db_conn();

        let (source_code, hardware_spec, program_spec): (
            String,
            models::HardwareSpec,
            models::ProgramSpec,
        ) = match user_programs::table
            .find(self.id)
            .filter(user_programs::columns::user_id.eq(user.id))
            .inner_join(program_specs::table.inner_join(hardware_specs::table))
            .select((
                user_programs::columns::source_code,
                hardware_specs::all_columns,
                program_specs::all_columns,
            ))
            .get_result(conn)
            .optional()?
        {
            // If the query returned no result, then the ID is no bueno. That
            // isn't a failure though, just return an empty response.
            None => return Ok(None),
            // query hit a result, continue
            Some(result) => result,
        };
        let program_spec_id = program_spec.id;

        // Compile and execute the program. If we get a GDLK error in either
        // case, then we want to return that but still consider the API response
        // a success (i.e. don't return a GQL error, return a "200" response
        // but with the compile/runtime error)

        let compiler =
            match Compiler::compile(source_code, hardware_spec.into()) {
                Ok(compiler) => compiler,
                // Compile error, return now
                Err(compile_error) => {
                    return Ok(Some(ExecuteUserProgramOutput::CompileError(
                        compile_error,
                    )))
                }
            };

        let mut machine = compiler.allocate(&program_spec.into());
        let successful = match machine.execute_all() {
            Ok(successful) => successful,
            // Runtime error, return now
            Err(runtime_error) => {
                return Ok(Some(ExecuteUserProgramOutput::RuntimeError(
                    runtime_error.clone(),
                )))
            }
        };

        let output = if successful {
            // Program succeeded, update the DB to store a record of this run
            let record = self.save_record(program_spec_id, &machine)?;
            ExecuteUserProgramOutput::Accepted { machine, record }
        } else {
            // No errors occurred, but the final state wasn't correct (input
            // still contained items, or output didn't match expectation)
            ExecuteUserProgramOutput::Rejected { machine }
        };
        Ok(Some(output))
    }
}
