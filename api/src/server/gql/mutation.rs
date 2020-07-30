use crate::{
    error::ResponseResult,
    server::gql::{
        Context, CopyUserProgramInput, CopyUserProgramPayload,
        CreateHardwareSpecInput, CreateHardwareSpecPayload,
        CreateProgramSpecInput, CreateProgramSpecPayload,
        CreateUserProgramInput, CreateUserProgramPayload,
        DeleteUserProgramInput, DeleteUserProgramPayload, InitializeUserInput,
        InitializeUserPayload, MutationFields, UpdateHardwareSpecInput,
        UpdateHardwareSpecPayload, UpdateProgramSpecInput,
        UpdateProgramSpecPayload, UpdateUserProgramInput,
        UpdateUserProgramPayload,
    },
    util,
    views::{self, View},
};
use juniper_from_schema::{QueryTrail, Walked};

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
        let view = views::InitializeUserView {
            conn: context.db_conn(),
            user_context: context.user_context,
            username: &input.username,
        };
        let created_user = view.execute()?;
        Ok(InitializeUserPayload { user: created_user })
    }

    fn field_create_hardware_spec(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, CreateHardwareSpecPayload, Walked>,
        input: CreateHardwareSpecInput,
    ) -> ResponseResult<CreateHardwareSpecPayload> {
        let context = executor.context();
        let view = views::CreateHardwareSpecView {
            conn: context.db_conn(),
            user_id: context.user_id()?,
            name: &input.name,
            num_registers: input.num_registers,
            num_stacks: input.num_stacks,
            max_stack_length: input.max_stack_length,
        };
        let hardware_spec = view.execute()?;

        Ok(CreateHardwareSpecPayload { hardware_spec })
    }

    fn field_update_hardware_spec(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, UpdateHardwareSpecPayload, Walked>,
        input: UpdateHardwareSpecInput,
    ) -> ResponseResult<UpdateHardwareSpecPayload> {
        let context = executor.context();
        let view = views::UpdateHardwareSpecView {
            conn: context.db_conn(),
            user_id: context.user_id()?,
            id: util::gql_id_to_uuid(&input.id),
            name: input.name.as_deref(),
            num_registers: input.num_registers,
            num_stacks: input.num_stacks,
            max_stack_length: input.max_stack_length,
        };
        let hardware_spec = view.execute()?;

        Ok(UpdateHardwareSpecPayload { hardware_spec })
    }

    fn field_create_program_spec(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, CreateProgramSpecPayload, Walked>,
        input: CreateProgramSpecInput,
    ) -> ResponseResult<CreateProgramSpecPayload> {
        let context = executor.context();
        let view = views::CreateProgramSpecView {
            conn: context.db_conn(),
            user_id: context.user_id()?,
            hardware_spec_id: util::gql_id_to_uuid(&input.hardware_spec_id),
            name: &input.name,
            description: &input.description,
            input: &input.input,
            expected_output: &input.expected_output,
        };
        let program_spec = view.execute()?;

        Ok(CreateProgramSpecPayload { program_spec })
    }

    fn field_update_program_spec(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, UpdateProgramSpecPayload, Walked>,
        input: UpdateProgramSpecInput,
    ) -> ResponseResult<UpdateProgramSpecPayload> {
        let context = executor.context();
        let view = views::UpdateProgramSpecView {
            conn: context.db_conn(),
            user_id: context.user_id()?,
            id: util::gql_id_to_uuid(&input.id),
            name: input.name.as_deref(),
            description: input.description.as_deref(),
            input: input.input.as_deref(),
            expected_output: input.expected_output.as_deref(),
        };
        let program_spec = view.execute()?;

        Ok(UpdateProgramSpecPayload { program_spec })
    }

    fn field_create_user_program(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, CreateUserProgramPayload, Walked>,
        input: CreateUserProgramInput,
    ) -> ResponseResult<CreateUserProgramPayload> {
        let context = executor.context();
        let view = views::CreateUserProgramView {
            conn: context.db_conn(),
            user_id: context.user_id()?,
            program_spec_id: util::gql_id_to_uuid(&input.program_spec_id),
            file_name: &input.file_name,
            // If no source is provided, default to an empty string
            source_code: input.source_code.as_deref().unwrap_or(""),
        };
        let user_program = view.execute()?;

        Ok(CreateUserProgramPayload { user_program })
    }

    fn field_update_user_program(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, UpdateUserProgramPayload, Walked>,
        input: UpdateUserProgramInput,
    ) -> ResponseResult<UpdateUserProgramPayload> {
        let context = executor.context();
        let view = views::UpdateUserProgramView {
            conn: context.db_conn(),
            user_id: context.user_id()?,
            id: util::gql_id_to_uuid(&input.id),
            file_name: input.file_name.as_deref(),
            source_code: input.source_code.as_deref(),
        };
        let user_program = view.execute()?;

        Ok(UpdateUserProgramPayload { user_program })
    }

    fn field_copy_user_program(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, CopyUserProgramPayload, Walked>,
        input: CopyUserProgramInput,
    ) -> ResponseResult<CopyUserProgramPayload> {
        let context = executor.context();
        let view = views::CopyUserProgramView {
            conn: context.db_conn(),
            user_id: context.user_id()?,
            id: util::gql_id_to_uuid(&input.id),
        };
        let user_program = view.execute()?;

        Ok(CopyUserProgramPayload { user_program })
    }

    fn field_delete_user_program(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, DeleteUserProgramPayload, Walked>,
        input: DeleteUserProgramInput,
    ) -> ResponseResult<DeleteUserProgramPayload> {
        let context = executor.context();
        let view = views::DeleteUserProgramView {
            conn: context.db_conn(),
            user_id: context.user_id()?,
            id: util::gql_id_to_uuid(&input.id),
        };
        let deleted_id = view.execute()?;

        Ok(DeleteUserProgramPayload { deleted_id })
    }
}
