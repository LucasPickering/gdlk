use crate::lang::StackIdentifier;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum CompileError {
    /// Failed to parse the program
    ParseError(String),
}

#[derive(Debug, Serialize)]
pub enum RuntimeError {
    /// Referenced a stack with an invalid identifier
    InvalidStackReference(StackIdentifier),

    /// Tried to push onto stack that is at capacity
    StackOverflow(StackIdentifier),

    /// READ attempted while input is empty
    EmptyInput,

    /// POP attempted while stack is empty
    EmptyStack(StackIdentifier),

    /// Instruction list has been exhausted, program is terminated
    ProgramTerminated,
}
