//! GQL types related to compiling/executing GDLK programs. This types do NOT
//! correspond to DB models.

use crate::{
    server::gql::{
        ProgramCompileErrorFields, ProgramFailureOutputFields,
        ProgramRuntimeErrorFields, ProgramSuccessOutputFields,
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
pub struct ProgramFailureOutput {}

impl ProgramFailureOutputFields for ProgramFailureOutput {
    fn field_todo(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
    ) -> i32 {
        0
    }
}

#[derive(Clone, Debug)]
pub struct ProgramSuccessOutput {}

impl ProgramSuccessOutputFields for ProgramSuccessOutput {
    fn field_todo(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
    ) -> i32 {
        0
    }
}
