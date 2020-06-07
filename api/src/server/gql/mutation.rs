use crate::{
    error::{DbErrorConverter, ResponseResult},
    models,
    schema::{hardware_specs, program_specs, user_programs, users},
    server::gql::{
        Context, CreateHardwareSpecInput, CreateHardwareSpecPayload,
        CreateProgramSpecInput, CreateProgramSpecPayload,
        CreateUserProgramInput, CreateUserProgramPayload,
        DeleteUserProgramInput, DeleteUserProgramPayload, MutationFields,
        UpdateHardwareSpecInput, UpdateHardwareSpecPayload,
        UpdateProgramSpecInput, UpdateProgramSpecPayload,
        UpdateUserProgramInput, UpdateUserProgramPayload,
    },
    util,
};
use diesel::{
    ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, RunQueryDsl,
    Table,
};
use juniper_from_schema::{QueryTrail, Walked};
use uuid::Uuid;
use validator::Validate;

/// The top-level mutation object.
pub struct Mutation;

impl MutationFields for Mutation {
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
        let conn: &PgConnection =
            &executor.context().get_db_conn()? as &PgConnection;
        let user_id: Uuid = models::User::tmp_user()
            .select(users::columns::id)
            .get_result(conn)?;

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
        let conn: &PgConnection =
            &executor.context().get_db_conn()? as &PgConnection;

        // User a helper type to do the insert
        let modified_user_program = models::ModifiedUserProgram {
            id: util::gql_id_to_uuid(&input.id),
            file_name: input.file_name.as_deref(),
            source_code: input.source_code.as_deref(),
        };
        modified_user_program.validate()?;

        // Update the row, returning the new value. If the row doesn't exist,
        // this will return None.
        let result: Result<Option<models::UserProgram>, _> =
            diesel::update(user_programs::table.find(modified_user_program.id))
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

    fn field_delete_user_program(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, DeleteUserProgramPayload, Walked>,
        input: DeleteUserProgramInput,
    ) -> ResponseResult<DeleteUserProgramPayload> {
        use self::user_programs::dsl::*;

        // Delete and get the ID back
        let row_id = util::gql_id_to_uuid(&input.user_program_id);
        let deleted_id: Option<Uuid> =
            diesel::delete(user_programs.filter(id.eq(row_id)))
                .returning(id)
                .get_result(&executor.context().get_db_conn()?)
                .optional()?;

        Ok(DeleteUserProgramPayload { deleted_id })
    }
}
