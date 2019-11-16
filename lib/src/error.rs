use crate::ast::StackIdentifier;
use failure::Fail;

#[derive(Debug, Fail)]
pub enum RuntimeError {
    /// Referenced a stack with an invalid identifier
    #[fail(display = "Invalid stack reference")]
    InvalidStackReference(StackIdentifier),

    /// Tried to push onto stack that is at capacity
    #[fail(display = "Stack overflow")]
    StackOverflow(StackIdentifier),

    /// READ attempted while input is empty
    #[fail(display = "Empty input")]
    EmptyInput,

    /// POP attempted while stack is empty
    #[fail(display = "Empty stack")]
    EmptyStack(StackIdentifier),
}
