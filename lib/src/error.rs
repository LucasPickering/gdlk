use crate::ast::StackIdentifier;
use failure::Fail;

#[derive(Debug, Fail)]
pub enum RuntimeError {
    /// Referenced a stack with an invalid identifier
    #[fail(display = "Invalid stack reference: {}", 0)]
    InvalidStackReference(StackIdentifier),

    /// Tried to push onto stack that is at capacity
    #[fail(display = "Overflow on stack {}", 0)]
    StackOverflow(StackIdentifier),

    /// READ attempted while input is empty
    #[fail(display = "Empty input")]
    EmptyInput,

    /// POP attempted while stack is empty
    #[fail(display = "Stack {} is empty", 0)]
    EmptyStack(StackIdentifier),
}
