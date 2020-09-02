use crate::{
    error::{DbErrorConverter, ResponseResult},
    models,
    schema::user_programs,
    views::{RequestContext, View},
};
use diesel::{OptionalExtension, RunQueryDsl, Table};
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
