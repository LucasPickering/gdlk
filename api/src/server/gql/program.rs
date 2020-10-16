//! GQL types related to compiling/executing GDLK programs. This types do NOT
//! correspond to DB models.

use crate::{
    server::gql::{
        MachineStateFields, ProgramAcceptedOutputFields,
        ProgramCompileErrorFields, ProgramErrorFields,
        ProgramRejectedOutputFields, ProgramRuntimeErrorFields,
        UserProgramRecordNode,
    },
    views::RequestContext,
};
use gdlk::{
    error::{SourceError, SourceErrorWrapper, WithSource},
    Machine,
};
use juniper_from_schema::{QueryTrail, Walked};
use std::convert::TryInto;

/// See description in schema.graphql
#[derive(Clone, Debug)]
pub struct ProgramError {
    pub message: String,
}

impl<E: SourceError> From<&SourceErrorWrapper<E>> for ProgramError {
    fn from(other: &SourceErrorWrapper<E>) -> Self {
        Self {
            message: other.to_string(),
        }
    }
}

impl ProgramError {
    pub fn from_source_error<E: SourceError>(
        error: &WithSource<E>,
    ) -> Vec<Self> {
        error.errors().iter().map(|err| err.into()).collect()
    }
}

impl ProgramErrorFields for ProgramError {
    fn field_message(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
    ) -> &String {
        &self.message
    }
}

/// See description in schema.graphql
#[derive(Clone, Debug)]
pub struct MachineState {
    pub machine: Machine,
}

impl From<Machine> for MachineState {
    fn from(machine: Machine) -> Self {
        Self { machine }
    }
}

impl MachineStateFields for MachineState {
    fn field_cpu_cycles(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
    ) -> i32 {
        // CPU cycles are capped at 1 million so easily in i32 range
        self.machine.cycle_count().try_into().unwrap()
    }
}

/// See description in schema.graphql
#[derive(Clone, Debug)]
pub struct ProgramCompileError {
    pub errors: Vec<ProgramError>,
}

impl ProgramCompileErrorFields for ProgramCompileError {
    fn field_errors(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
        _trail: &QueryTrail<'_, ProgramError, Walked>,
    ) -> &Vec<ProgramError> {
        &self.errors
    }
}

/// See description in schema.graphql
#[derive(Clone, Debug)]
pub struct ProgramRuntimeError {
    pub error: ProgramError,
}

impl ProgramRuntimeErrorFields for ProgramRuntimeError {
    fn field_error(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
        _trail: &QueryTrail<'_, ProgramError, Walked>,
    ) -> &ProgramError {
        &self.error
    }
}

/// See description in schema.graphql
#[derive(Clone, Debug)]
pub struct ProgramRejectedOutput {
    pub machine_state: MachineState,
}

impl ProgramRejectedOutputFields for ProgramRejectedOutput {
    fn field_machine(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
        _trail: &QueryTrail<'_, MachineState, Walked>,
    ) -> &MachineState {
        &self.machine_state
    }
}

/// See description in schema.graphql
#[derive(Clone, Debug)]
pub struct ProgramAcceptedOutput {
    pub machine_state: MachineState,
    pub user_program_record: UserProgramRecordNode,
}

impl ProgramAcceptedOutputFields for ProgramAcceptedOutput {
    fn field_machine(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
        _trail: &QueryTrail<'_, MachineState, Walked>,
    ) -> &MachineState {
        &self.machine_state
    }

    fn field_record(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
        _trail: &QueryTrail<'_, UserProgramRecordNode, Walked>,
    ) -> &UserProgramRecordNode {
        &self.user_program_record
    }
}
