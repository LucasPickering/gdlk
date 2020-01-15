//! Error types and other error-related code.

use actix_web::{HttpResponse, ResponseError};
use failure::Fail;

pub type Result<T> = std::result::Result<T, ServerError>;

#[derive(Debug, Fail)]
pub enum ServerError {
    /// Wrapper for R2D2's error type
    #[fail(display = "{}", 0)]
    R2d2Error(#[cause] r2d2::Error),
    /// Wrapper for Diesel's error type
    #[fail(display = "{}", 0)]
    DieselError(#[cause] diesel::result::Error),
    #[fail(display = "File or directory not found")]
    NodeNotFound,
    #[fail(display = "Operation not supported")]
    UnsupportedFileOperation,
}

impl From<r2d2::Error> for ServerError {
    fn from(other: r2d2::Error) -> Self {
        Self::R2d2Error(other)
    }
}

impl From<diesel::result::Error> for ServerError {
    fn from(other: diesel::result::Error) -> Self {
        Self::DieselError(other)
    }
}

impl ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse {
        match self {
            // 400
            Self::UnsupportedFileOperation => HttpResponse::BadRequest().into(),
            // 404
            Self::DieselError(diesel::result::Error::NotFound)
            | Self::NodeNotFound => HttpResponse::NotFound().into(),
            // 500
            _ => HttpResponse::InternalServerError().into(),
        }
    }
}
