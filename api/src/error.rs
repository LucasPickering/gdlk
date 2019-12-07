use crate::lang::{RegisterRef, StackIdentifier, UserRegisterIdentifier};
use actix_web::{HttpResponse, ResponseError};
use failure::Fail;
use serde::Serialize;

#[derive(Debug, Fail, Serialize)]
pub enum CompileError {
    /// Failed to parse the program
    #[fail(display = "Parse error: {}", 0)]
    ParseError(String),
}

#[derive(Debug, Fail, Serialize)]
pub enum RuntimeError {
    /// Referenced a user register with an invalid identifier
    #[fail(display = "Invalid reference to register {}", 0)]
    InvalidUserRegisterRef(UserRegisterIdentifier),

    /// Referenced a stack with an invalid identifier
    #[fail(display = "Invalid reference to stack {}", 0)]
    InvalidStackRef(StackIdentifier),

    /// Tried to write to a read-only register
    #[fail(display = "Cannot write to read-only register {}", 0)]
    UnwritableRegister(RegisterRef),

    /// Tried to push onto stack that is at capacity
    #[fail(display = "Overflow on stack {}", 0)]
    StackOverflow(StackIdentifier),

    /// READ attempted while input is empty
    #[fail(display = "No input available to read")]
    EmptyInput,

    /// POP attempted while stack is empty
    #[fail(display = "Cannot pop from empty stack {}", 0)]
    EmptyStack(StackIdentifier),

    /// Instruction list has been exhausted, program is terminated
    #[fail(display = "Program has terminated, nothing left to execute")]
    ProgramTerminated,
}

#[derive(Debug, Fail)]
pub enum ServerError {
    #[fail(display = "{}", 0)]
    DbError(#[cause] diesel::result::Error),
}

impl From<diesel::result::Error> for ServerError {
    fn from(other: diesel::result::Error) -> Self {
        Self::DbError(other)
    }
}

impl ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::DbError(diesel::result::Error::NotFound) => {
                HttpResponse::NotFound().into()
            }
            _ => HttpResponse::InternalServerError().into(),
        }
    }
}
