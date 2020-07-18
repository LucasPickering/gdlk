use crate::{
    error::{DbErrorConverter, ResponseError, ResponseResult},
    models,
    schema::{
        hardware_specs, program_specs, user_programs, user_providers, users,
    },
    server::gql::{
        Context, CopyUserProgramInput, CopyUserProgramPayload,
        CreateHardwareSpecInput, CreateHardwareSpecPayload,
        CreateProgramSpecInput, CreateProgramSpecPayload,
        CreateUserProgramInput, CreateUserProgramPayload,
        DeleteUserProgramInput, DeleteUserProgramPayload, InitializeUserInput,
        InitializeUserPayload, MutationFields, UpdateHardwareSpecInput,
        UpdateHardwareSpecPayload, UpdateProgramSpecInput,
        UpdateProgramSpecPayload, UpdateUserProgramInput,
        UpdateUserProgramPayload, UserContext,
    },
    util,
};
use diesel::{
    Connection, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl,
    RunQueryDsl, Table,
};
use juniper_from_schema::{QueryTrail, Walked};
use uuid::Uuid;
use validator::Validate;

/// The top-level mutation object.
pub struct Mutation;

impl MutationFields for Mutation {
    fn field_initialize_user(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, InitializeUserPayload, Walked>,
        input: InitializeUserInput,
    ) -> ResponseResult<InitializeUserPayload> {
        let context = executor.context();

        // The user should be logged in, but not had a User object created yet
        match context.user_context {
            Some(UserContext {
                user_provider_id, ..
            }) => {
                let conn = &context.get_db_conn()?;
                let new_user = models::NewUser {
                    username: &input.username,
                };
                new_user.validate()?;

                // We need to insert the new user row, then update the
                // user_provider to point at that row. We need a transaction to
                // prevent race conditions.
                let created_user = conn
                    .transaction::<models::User, ResponseError, _>(|| {
                        let create_user_result: Result<models::User, _> =
                            new_user
                                .insert()
                                .returning(users::all_columns)
                                .get_result(conn);

                        // Check if the username already exists
                        let created_user = DbErrorConverter {
                            unique_violation_to_exists: true,
                            ..Default::default()
                        }
                        .convert(create_user_result)?;

                        // We should update exactly 1 row. If not, then either
                        // the referenced user_provider row is already linked to
                        // a user, or it doesn't exist. In either case, just
                        // return a NotFound error.
                        let updated_rows = diesel::update(
                            user_providers::table
                                .find(user_provider_id)
                                .filter(
                                    user_providers::columns::user_id.is_null(),
                                ),
                        )
                        .set(
                            user_providers::columns::user_id
                                .eq(Some(created_user.id)),
                        )
                        .execute(conn)?;

                        if updated_rows == 0 {
                            Err(ResponseError::NotFound)
                        } else {
                            Ok(created_user)
                        }
                    })?;

                Ok(InitializeUserPayload { user: created_user })
            }
            // Get up on outta here
            None => Err(ResponseError::Unauthenticated),
        }
    }

    fn field_create_hardware_spec(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, CreateHardwareSpecPayload, Walked>,
        input: CreateHardwareSpecInput,
    ) -> ResponseResult<CreateHardwareSpecPayload> {
        let conn: &PgConnection =
            &executor.context().get_db_conn()? as &PgConnection;

        // User a helper type to do the insert
        let new_hardware_spec = models::NewHardwareSpec {
            name: &input.name,
            num_registers: input.num_registers,
            num_stacks: input.num_stacks,
            max_stack_length: input.max_stack_length,
        };
        new_hardware_spec.validate()?;

        // Insert the new row and return the whole row
        let result: Result<models::HardwareSpec, _> = new_hardware_spec
            .insert()
            .returning(hardware_specs::table::all_columns())
            .get_result(conn);

        let hardware_spec: models::HardwareSpec = DbErrorConverter {
            // HardwareSpec already exists with this name or slug
            unique_violation_to_exists: true,
            ..Default::default()
        }
        .convert(result)?;

        Ok(CreateHardwareSpecPayload { hardware_spec })
    }

    fn field_update_hardware_spec(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, UpdateHardwareSpecPayload, Walked>,
        input: UpdateHardwareSpecInput,
    ) -> ResponseResult<UpdateHardwareSpecPayload> {
        let conn: &PgConnection =
            &executor.context().get_db_conn()? as &PgConnection;

        // User a helper type to do the insert
        let modified_hardware_spec = models::ModifiedHardwareSpec {
            id: util::gql_id_to_uuid(&input.id),
            name: input.name.as_deref(),
            num_registers: input.num_registers,
            num_stacks: input.num_stacks,
            max_stack_length: input.max_stack_length,
        };
        modified_hardware_spec.validate()?;

        // Update the row, returning the new value. If the row doesn't exist,
        // this will return None.
        let result: Result<Option<models::HardwareSpec>, _> = diesel::update(
            hardware_specs::table.find(modified_hardware_spec.id),
        )
        .set(modified_hardware_spec)
        .returning(hardware_specs::table::all_columns())
        .get_result(conn)
        .optional();

        let updated_row: Option<models::HardwareSpec> = DbErrorConverter {
            // HardwareSpec already exists with this name or slug
            unique_violation_to_exists: true,
            // No update fields were given
            query_builder_to_no_update: true,
            ..Default::default()
        }
        .convert(result)?;

        Ok(UpdateHardwareSpecPayload {
            hardware_spec: updated_row,
        })
    }

    fn field_create_program_spec(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, CreateProgramSpecPayload, Walked>,
        input: CreateProgramSpecInput,
    ) -> ResponseResult<CreateProgramSpecPayload> {
        let conn: &PgConnection =
            &executor.context().get_db_conn()? as &PgConnection;

        // User a helper type to do the insert
        let new_program_spec = models::NewProgramSpec {
            hardware_spec_id: util::gql_id_to_uuid(&input.hardware_spec_id),
            name: &input.name,
            description: &input.description,
            input: input.input,
            expected_output: input.expected_output,
        };
        new_program_spec.validate()?;

        // Insert the new row and return the whole row
        let result: Result<models::ProgramSpec, _> = new_program_spec
            .insert()
            .returning(program_specs::table::all_columns())
            .get_result(conn);

        let program_spec: models::ProgramSpec = DbErrorConverter {
            // Given hardware spec ID was invalid
            fk_violation_to_not_found: true,
            // ProgramSpec already exists with this name or slug
            unique_violation_to_exists: true,
            ..Default::default()
        }
        .convert(result)?;

        Ok(CreateProgramSpecPayload { program_spec })
    }

    fn field_update_program_spec(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, UpdateProgramSpecPayload, Walked>,
        input: UpdateProgramSpecInput,
    ) -> ResponseResult<UpdateProgramSpecPayload> {
        let conn: &PgConnection =
            &executor.context().get_db_conn()? as &PgConnection;

        // Use a helper type to do the insert
        let modified_program_spec = models::ModifiedProgramSpec {
            id: util::gql_id_to_uuid(&input.id),
            name: input.name.as_deref(),
            description: input.description.as_deref(),
            input: input.input,
            expected_output: input.expected_output,
        };
        modified_program_spec.validate()?;

        // Update the row, returning the new value. If the row doesn't exist,
        // this will return None.
        let result: Result<Option<models::ProgramSpec>, _> =
            diesel::update(program_specs::table.find(modified_program_spec.id))
                .set(modified_program_spec)
                .returning(program_specs::table::all_columns())
                .get_result(conn)
                .optional();

        let updated_row: Option<models::ProgramSpec> = DbErrorConverter {
            // ProgramSpec already exists with this name or slug
            unique_violation_to_exists: true,
            // No update fields were given
            query_builder_to_no_update: true,
            ..Default::default()
        }
        .convert(result)?;

        Ok(UpdateProgramSpecPayload {
            program_spec: updated_row,
        })
    }

    fn field_create_user_program(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, CreateUserProgramPayload, Walked>,
        input: CreateUserProgramInput,
    ) -> ResponseResult<CreateUserProgramPayload> {
        let context = executor.context();
        let conn: &PgConnection = &context.get_db_conn()? as &PgConnection;
        let user_id = context.user_id()?; // User needs to be logged in

        // User a helper type to do the insert
        let new_user_program = models::NewUserProgram {
            user_id,
            program_spec_id: util::gql_id_to_uuid(&input.program_spec_id),
            file_name: &input.file_name,
            // If no source is provided, default to an empty string
            source_code: input.source_code.as_deref().unwrap_or(""),
        };
        new_user_program.validate()?;

        // Insert the new row and return the whole row
        let result: Result<models::UserProgram, _> = new_user_program
            .insert()
            .returning(user_programs::table::all_columns())
            .get_result(conn);

        let user_program: models::UserProgram = DbErrorConverter {
            // Given program spec ID was invalid. Note: this can also indicate
            // an invalid user ID which would be a server-side bug, but fuck it
            fk_violation_to_not_found: true,
            // UserProgram already exists with this program spec/file name
            unique_violation_to_exists: true,
            ..Default::default()
        }
        .convert(result)?;

        Ok(CreateUserProgramPayload { user_program })
    }

    fn field_update_user_program(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, UpdateUserProgramPayload, Walked>,
        input: UpdateUserProgramInput,
    ) -> ResponseResult<UpdateUserProgramPayload> {
        let context = executor.context();
        let conn: &PgConnection = &context.get_db_conn()? as &PgConnection;
        // User needs to be logged in to make changes. This also has to be
        // their user_program.
        let user_id = context.user_id()?;

        // User a helper type to do the insert
        let modified_user_program = models::ModifiedUserProgram {
            id: util::gql_id_to_uuid(&input.id),
            file_name: input.file_name.as_deref(),
            source_code: input.source_code.as_deref(),
        };
        modified_user_program.validate()?;

        // Update the row, returning the new value. If the row doesn't
        // exist, this will return None.
        let result: Result<Option<models::UserProgram>, _> =
            diesel::update(models::UserProgram::find_for_user(
                modified_user_program.id,
                user_id,
            ))
            .set(modified_user_program)
            .returning(user_programs::table::all_columns())
            .get_result(conn)
            .optional();

        let updated_row: Option<models::UserProgram> = DbErrorConverter {
            // UserProgram already exists with this program spec/file name
            unique_violation_to_exists: true,
            // No update fields were given
            query_builder_to_no_update: true,
            ..Default::default()
        }
        .convert(result)?;

        Ok(UpdateUserProgramPayload {
            user_program: updated_row,
        })
    }

    fn field_copy_user_program(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, CopyUserProgramPayload, Walked>,
        input: CopyUserProgramInput,
    ) -> ResponseResult<CopyUserProgramPayload> {
        let context = executor.context();
        let conn: &PgConnection = &context.get_db_conn()? as &PgConnection;
        // User needs to be logged in to make changes. This also has to be
        // their user_program.
        let user_id = context.user_id()?;

        let existing_user_program: Option<models::UserProgram> =
            models::UserProgram::find_for_user(
                util::gql_id_to_uuid(&input.id),
                user_id,
            )
            .get_result(conn)
            .optional()?;

        // If the requested user_program exists (for the given user), copy it
        let inserted_row = match existing_user_program {
            None => None,
            Some(user_program) => {
                Some(
                    models::NewUserProgram {
                        user_id,
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
        };

        Ok(CopyUserProgramPayload {
            user_program: inserted_row,
        })
    }

    fn field_delete_user_program(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, DeleteUserProgramPayload, Walked>,
        input: DeleteUserProgramInput,
    ) -> ResponseResult<DeleteUserProgramPayload> {
        let context = executor.context();
        let conn: &PgConnection = &context.get_db_conn()? as &PgConnection;
        // User needs to be logged in to make changes. This also has to be
        // their user_program.
        let user_id = context.user_id()?;

        // Delete and get the ID back
        let row_id = util::gql_id_to_uuid(&input.id);
        let deleted_id: Option<Uuid> =
            diesel::delete(models::UserProgram::find_for_user(row_id, user_id))
                .returning(user_programs::columns::id)
                .get_result(conn)
                .optional()?;

        Ok(DeleteUserProgramPayload { deleted_id })
    }
}
