//! GQL types related to compiling/executing GDLK programs. This types do NOT
//! correspond to DB models.

use crate::{
    server::gql::{
        ProgramAcceptedOutputFields, ProgramCompileErrorFields,
        ProgramRejectedOutputFields, ProgramRuntimeErrorFields,
    },
    views::RequestContext,
};

// TODO fill out these fields properly

// #[derive(Clone, Debug)]
// pub struct ProgramError {}

// impl ProgramErrorFields for ProgramError {
//     fn field_todo(
//         &self,
//         _executor: &juniper::Executor<'_, RequestContext>,
//     ) -> i32 {
//         0
//     }
// }

// #[derive(Clone, Debug)]
// pub struct MachineState {}

// impl MachineStateFields for MachineState {
//     fn field_todo(
//         &self,
//         _executor: &juniper::Executor<'_, RequestContext>,
//     ) -> i32 {
//         0
//     }
// }

#[derive(Clone, Debug)]
pub struct ProgramCompileError {}

impl ProgramCompileErrorFields for ProgramCompileError {
    fn field_todo(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
    ) -> i32 {
        0
    }
}

#[derive(Clone, Debug)]
pub struct ProgramRuntimeError {}

impl ProgramRuntimeErrorFields for ProgramRuntimeError {
    fn field_todo(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
    ) -> i32 {
        0
    }
}

#[derive(Clone, Debug)]
pub struct ProgramRejectedOutput {}

impl ProgramRejectedOutputFields for ProgramRejectedOutput {
    fn field_todo(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
    ) -> i32 {
        0
    }
}

#[derive(Clone, Debug)]
pub struct ProgramAcceptedOutput {}

impl ProgramAcceptedOutputFields for ProgramAcceptedOutput {
    fn field_todo(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
    ) -> i32 {
        0
    }
}
