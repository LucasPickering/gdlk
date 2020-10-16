use crate::{
    error::ResponseResult,
    server::gql::{
        program::ProgramError, CopyUserProgramInput, CopyUserProgramPayload,
        CreateHardwareSpecInput, CreateHardwareSpecPayload,
        CreateProgramSpecInput, CreateProgramSpecPayload,
        CreateUserProgramInput, CreateUserProgramPayload,
        DeleteUserProgramInput, DeleteUserProgramPayload,
        ExecuteUserProgramInput, ExecuteUserProgramPayload,
        ExecuteUserProgramStatus, InitializeUserInput, InitializeUserPayload,
        MutationFields, ProgramAcceptedOutput, ProgramCompileError,
        ProgramRejectedOutput, ProgramRuntimeError, UpdateHardwareSpecInput,
        UpdateHardwareSpecPayload, UpdateProgramSpecInput,
        UpdateProgramSpecPayload, UpdateUserProgramInput,
        UpdateUserProgramPayload,
    },
    util,
    views::{self, ExecuteUserProgramOutput, RequestContext, View},
};
use juniper_from_schema::{QueryTrail, Walked};

/// The top-level mutation object.
pub struct Mutation;

impl MutationFields for Mutation {
    fn field_initialize_user(
        &self,
        executor: &juniper::Executor<'_, RequestContext>,
        _trail: &QueryTrail<'_, InitializeUserPayload, Walked>,
        input: InitializeUserInput,
    ) -> ResponseResult<InitializeUserPayload> {
        let view = views::InitializeUserView {
            context: executor.context(),
            username: &input.username,
        };
        let created_user = view.execute()?;
        Ok(InitializeUserPayload { user: created_user })
    }

    fn field_create_hardware_spec(
        &self,
        executor: &juniper::Executor<'_, RequestContext>,
        _trail: &QueryTrail<'_, CreateHardwareSpecPayload, Walked>,
        input: CreateHardwareSpecInput,
    ) -> ResponseResult<CreateHardwareSpecPayload> {
        let view = views::CreateHardwareSpecView {
            context: executor.context(),
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
        executor: &juniper::Executor<'_, RequestContext>,
        _trail: &QueryTrail<'_, UpdateHardwareSpecPayload, Walked>,
        input: UpdateHardwareSpecInput,
    ) -> ResponseResult<UpdateHardwareSpecPayload> {
        let view = views::UpdateHardwareSpecView {
            context: executor.context(),
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
        executor: &juniper::Executor<'_, RequestContext>,
        _trail: &QueryTrail<'_, CreateProgramSpecPayload, Walked>,
        input: CreateProgramSpecInput,
    ) -> ResponseResult<CreateProgramSpecPayload> {
        let view = views::CreateProgramSpecView {
            context: executor.context(),
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
        executor: &juniper::Executor<'_, RequestContext>,
        _trail: &QueryTrail<'_, UpdateProgramSpecPayload, Walked>,
        input: UpdateProgramSpecInput,
    ) -> ResponseResult<UpdateProgramSpecPayload> {
        let view = views::UpdateProgramSpecView {
            context: executor.context(),
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
        executor: &juniper::Executor<'_, RequestContext>,
        _trail: &QueryTrail<'_, CreateUserProgramPayload, Walked>,
        input: CreateUserProgramInput,
    ) -> ResponseResult<CreateUserProgramPayload> {
        let view = views::CreateUserProgramView {
            context: executor.context(),
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
        executor: &juniper::Executor<'_, RequestContext>,
        _trail: &QueryTrail<'_, UpdateUserProgramPayload, Walked>,
        input: UpdateUserProgramInput,
    ) -> ResponseResult<UpdateUserProgramPayload> {
        let view = views::UpdateUserProgramView {
            context: executor.context(),
            id: util::gql_id_to_uuid(&input.id),
            file_name: input.file_name.as_deref(),
            source_code: input.source_code.as_deref(),
        };
        let user_program = view.execute()?;

        Ok(UpdateUserProgramPayload { user_program })
    }

    fn field_copy_user_program(
        &self,
        executor: &juniper::Executor<'_, RequestContext>,
        _trail: &QueryTrail<'_, CopyUserProgramPayload, Walked>,
        input: CopyUserProgramInput,
    ) -> ResponseResult<CopyUserProgramPayload> {
        let view = views::CopyUserProgramView {
            context: executor.context(),
            id: util::gql_id_to_uuid(&input.id),
        };
        let user_program = view.execute()?;

        Ok(CopyUserProgramPayload { user_program })
    }

    fn field_delete_user_program(
        &self,
        executor: &juniper::Executor<'_, RequestContext>,
        _trail: &QueryTrail<'_, DeleteUserProgramPayload, Walked>,
        input: DeleteUserProgramInput,
    ) -> ResponseResult<DeleteUserProgramPayload> {
        let view = views::DeleteUserProgramView {
            context: executor.context(),
            id: util::gql_id_to_uuid(&input.id),
        };
        let deleted_id = view.execute()?;

        Ok(DeleteUserProgramPayload { deleted_id })
    }

    fn field_execute_user_program(
        &self,
        executor: &juniper::Executor<'_, RequestContext>,
        _trail: &QueryTrail<'_, ExecuteUserProgramPayload, Walked>,
        input: ExecuteUserProgramInput,
    ) -> ResponseResult<ExecuteUserProgramPayload> {
        let view = views::ExecuteUserProgramView {
            context: executor.context(),
            id: util::gql_id_to_uuid(&input.id),
        };
        // Map the status type to our GQL output type
        let status = view.execute()?.map(|output| match output {
            ExecuteUserProgramOutput::CompileError(error) => {
                ExecuteUserProgramStatus::ProgramCompileError(
                    ProgramCompileError {
                        errors: ProgramError::from_source_error(&error),
                    },
                )
            }
            ExecuteUserProgramOutput::RuntimeError(error) => {
                ExecuteUserProgramStatus::ProgramRuntimeError(
                    ProgramRuntimeError {
                        error: error.errors().first().unwrap().into(),
                    },
                )
            }
            ExecuteUserProgramOutput::Rejected { machine } => {
                ExecuteUserProgramStatus::ProgramRejectedOutput(
                    ProgramRejectedOutput {
                        machine_state: machine.into(),
                    },
                )
            }
            ExecuteUserProgramOutput::Accepted { machine, record } => {
                ExecuteUserProgramStatus::ProgramAcceptedOutput(
                    ProgramAcceptedOutput {
                        machine_state: machine.into(),
                        user_program_record: record.into(),
                    },
                )
            }
        });
        Ok(ExecuteUserProgramPayload { status })
    }
}
