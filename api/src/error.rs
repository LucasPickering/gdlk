use crate::lang::StackIdentifier;
use actix_web::{HttpResponse, ResponseError};
use failure::Fail;
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
