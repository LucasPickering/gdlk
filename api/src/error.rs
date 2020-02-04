//! Error types and other error-related code.

use actix_web::{HttpResponse, ResponseError};
use failure::Fail;
use juniper::{FieldError, IntoFieldError, Value};

pub type Result<T> = std::result::Result<T, ServerError>;

#[derive(Debug, Fail)]
pub enum ServerError {
    /// Wrapper for R2D2's error type
    #[fail(display = "{}", 0)]
    R2d2Error(#[cause] r2d2::Error),

    /// Wrapper for Diesel's error type
    #[fail(display = "{}", 0)]
    DieselError(#[cause] diesel::result::Error),

    // ===== FS errors =====
    /// A file system node was requested, but no node exists at the given path.
    #[fail(display = "File or directory not found")]
    NodeNotFound,

    /// The attempted file system operation is not supported for the node it
    /// was attempted on.
    #[fail(display = "Operation not supported")]
    UnsupportedFileOperation,

    /// The file system node does not provide the permissions required to
    /// perform the attempted operation.
    #[fail(display = "Permission denied")]
    PermissionDenied,

    /// User attempted to create a node, but a node already exists at that
    /// path.
    #[fail(display = "Node already exists")]
    AlreadyExists,
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

// Juniper error
impl IntoFieldError for ServerError {
    fn into_field_error(self) -> FieldError {
        FieldError::new(format!("{}", self), Value::Null)
    }
}

// Actix error
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
