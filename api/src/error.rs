//! Error types and other error-related code.

use crate::util;
use actix_web::{
    client::{PayloadError, SendRequestError},
    HttpResponse,
};
use diesel::result::DatabaseErrorKind;
use juniper::{DefaultScalarValue, FieldError, IntoFieldError};
use log::{error, log, Level};
use openidconnect::{core::CoreErrorResponseType, StandardErrorResponse};
use std::{
    array::TryFromSliceError, backtrace::Backtrace, error::Error, fmt::Debug,
};
use thiserror::Error;
use validator::{ValidationError, ValidationErrors, ValidationErrorsKind};

pub type ResponseResult<T> = Result<T, ResponseError>;

/// TODO
#[derive(Debug, Error)]
pub enum ClientError {
    /// User tried to tried to reference a non-existent resource. Be careful
    /// with this! This should NOT be to respond to queries where the missing
    /// resource was directly queried. E.g. if querying hardware specs by slug,
    /// and there is no row with the given slug, the API should return `None`,
    /// NOT this variant! This should be returned when the user implicitly
    /// assumes a resource exists when it does not. For example, insert a new
    /// row and specifying a FK to a related row. If that FK is invalid, that
    /// would be a good time to return this variant.
    #[error("Not found")]
    NotFound {
        /// The diesel error that triggered this, used just for debugging.
        /// Should be provided whenever possible.
        source: Option<diesel::result::Error>,
    },

    /// User tried to use some unique identifier that already exists. This
    /// could occur during a create, rename, etc.
    #[error("This resource already exists")]
    AlreadyExists {
        /// The diesel error that triggered this, used just for debugging
        source: diesel::result::Error,
    },

    /// User tried to perform an update mutation, but didn't given any values
    /// to change.
    #[error("No fields were given to update")]
    NoUpdate {
        /// The diesel error that triggered this, used just for debugging
        source: diesel::result::Error,
    },

    /// Action cannot be performed because the user is not authenticated.
    #[error("Not logged in")]
    Unauthenticated,

    /// Action cannot be performed because the user doesn't have permission to
    /// do so. This should only be used when the user is already logged in.
    /// Equivalent of an HTTP 403.
    #[error("Insufficient permissions to perform this action")]
    PermissionDenied,

    /// CSRF failure during auth
    #[error("CSRF token was not provided or did not match the expected value")]
    CsrfError,

    /// Claims returned from the OpenID provider are invalid in some way
    #[error("Claims verification failure: {0}")]
    ClaimsVerificationError(#[from] openidconnect::ClaimsVerificationError),

    /// Wrapper for a serde_json error.
    #[error("Serialization/deserialization error: {}", 0)]
    SerializationError(#[from] serde_json::error::Error),

    /// Wrapper for validator's error type
    #[error("Validator error: {}", 0)]
    ValidationErrors(#[from] ValidationErrors),

    /// Wrapper for an OpenID token error, which can occur while validating a
    /// token submitted by a user.
    #[error("{0}")]
    RequestTokenError(
        #[from]
        openidconnect::RequestTokenError<
            ActixClientError,
            StandardErrorResponse<CoreErrorResponseType>,
        >,
    ),
}

/// TODO
#[derive(Debug, Error)]
pub enum ServerError {
    /// Wrapper for R2D2's error type
    #[error("Database error: {0}")]
    R2d2Error(#[from] r2d2::Error),

    /// Wrapper for Diesel's error type
    #[error("Database error: {0}")]
    DieselError(#[from] diesel::result::Error),

    /// When we do token exchange with an OpenID provider, we always expect to
    /// get an `id_token` field back in the response. If we don't for some
    /// reason (either we fucked up or the provider fucked up), use this error.
    #[error("id_token field was not in OpenID response")]
    MissingIdToken,
}

/// An error that can occur while handling an HTTP request. These errors should
/// all at least somewhat meaningful to the user.
#[derive(Debug, Error)]
pub enum ResponseError {
    #[error("{source}")]
    Client {
        source: ClientError,
        backtrace: Backtrace,
    },
    #[error("{source}")]
    Server {
        source: ServerError,
        backtrace: Backtrace,
    },
}

impl ResponseError {
    /// TODO
    pub fn from_client_error<T: Into<ClientError>>(source: T) -> Self {
        let e: ClientError = source.into();
        e.into()
    }

    /// TODO
    pub fn from_server_error<T: Into<ServerError>>(source: T) -> Self {
        let e: ServerError = source.into();
        e.into()
    }

    /// Log this error to the server output. Server errors use the `error` log
    /// level, while client errors use `debug`.
    pub fn log(&self) {
        let log_level = match self {
            ResponseError::Client { .. } => Level::Debug,
            ResponseError::Server { .. } => Level::Error,
        };

        // Print both the Display and the Debug version, for completeness
        log!(
            log_level,
            "Response Error: {}\n{:#?}\n{}",
            self,
            // ResponseError's type enforces that source and backtrace are
            // always present
            self.source().unwrap(),
            self.backtrace().expect("No backtrace available :(")
        );
    }
}

// These two implementations are the only way that `ResponseError`s should be
// created, to ensure they get the backtrace.
impl From<ClientError> for ResponseError {
    fn from(source: ClientError) -> Self {
        Self::Client {
            source,
            backtrace: Backtrace::capture(),
        }
    }
}
impl From<ServerError> for ResponseError {
    fn from(source: ServerError) -> Self {
        Self::Server {
            source,
            backtrace: Backtrace::capture(),
        }
    }
}

// Implementations for some COMMON and UNAMBIGUOUS error conversions. If an
// error type could concievably be both a client and a server error, then we
// shouldn't implement these conversions, to prevent accidental mis-conversions.
impl From<ValidationErrors> for ResponseError {
    fn from(source: ValidationErrors) -> Self {
        let e: ClientError = source.into();
        e.into()
    }
}
impl From<diesel::result::Error> for ResponseError {
    fn from(source: diesel::result::Error) -> Self {
        let e: ServerError = source.into();
        e.into()
    }
}

// Juniper error
impl IntoFieldError for ResponseError {
    fn into_field_error(self) -> FieldError {
        // Temporary method to log errors
        // TODO https://github.com/LucasPickering/gdlk/issues/125
        self.log();

        match self {
            ResponseError::Client {
                source: ClientError::ValidationErrors(errors),
                ..
            } => validation_to_field_error(errors),
            _ => FieldError::new(self.to_string(), juniper::Value::Null),
        }
    }
}

// Actix error
impl actix_web::ResponseError for ResponseError {
    fn error_response(&self) -> HttpResponse {
        // Temporary method to log errors
        // TODO https://github.com/LucasPickering/gdlk/issues/125
        self.log();

        match self {
            Self::Client { source, .. } => {
                match source {
                    // 401
                    ClientError::CsrfError
                    | ClientError::ClaimsVerificationError(_) => {
                        HttpResponse::Unauthorized().into()
                    }
                    // 403
                    ClientError::PermissionDenied => {
                        HttpResponse::Forbidden().into()
                    }
                    // 409
                    ClientError::AlreadyExists { .. } => {
                        HttpResponse::Conflict().into()
                    }
                    // Everything else becomes a 400
                    _ => HttpResponse::BadRequest().into(),
                }
            }
            Self::Server { source, .. } => {
                match source {
                    // 503
                    ServerError::R2d2Error(_) => {
                        HttpResponse::ServiceUnavailable().into()
                    }
                    // Everything else becomes a 500
                    _ => HttpResponse::InternalServerError().into(),
                }
            }
        }
    }
}

/// Converts a [ValidationErrors] to a [FieldError]. Useful for validating input
/// objects in GraphQL responders.
fn validation_to_field_error(errors: ValidationErrors) -> FieldError {
    /// Convert a singular error to a GQL object.
    fn convert_single_error(error: ValidationError) -> juniper::Value {
        // Convert the individual error params to GQL strings, then build them
        // into an object.
        util::map_to_gql_object(error.params.into_iter(), |value| {
            juniper::Value::Scalar(DefaultScalarValue::String(
                value.to_string(),
            ))
        })
    }

    /// Convert a collection of errors to a GQL object with nested values.
    fn convert_errors(errors: ValidationErrors) -> juniper::Value {
        util::map_to_gql_object(errors.into_errors().into_iter(), |value| {
            match value {
                ValidationErrorsKind::Struct(child_errors) => {
                    convert_errors(*child_errors)
                }
                ValidationErrorsKind::List(error_map) => {
                    util::map_to_gql_object(error_map.into_iter(), |errors| {
                        convert_errors(*errors)
                    })
                }
                ValidationErrorsKind::Field(field_errors) => {
                    let converted_errs: Vec<juniper::Value<_>> = field_errors
                        .into_iter()
                        .map(convert_single_error)
                        .collect();
                    juniper::Value::List(converted_errs)
                }
            }
        })
    }

    FieldError::new("Input validation error(s)", convert_errors(errors))
}

/// An error that can occur during a HTTP call via the Actix client.
#[derive(Debug, Error)]
pub enum ActixClientError {
    /// A wrapper for [actix_web::client::SendRequestError]. This type doesn't
    /// implement `Error` so we have to convert it to a string first.
    #[error("{0}")]
    SendRequestError(String),

    /// A wrapper for [actix_web::client::PayloadError]. This type doesn't
    /// implement `Error` so we have to convert it to a string first.
    #[error("{0}")]
    PayloadError(String),
}

impl From<SendRequestError> for ActixClientError {
    fn from(other: SendRequestError) -> Self {
        Self::SendRequestError(other.to_string())
    }
}

impl From<PayloadError> for ActixClientError {
    fn from(other: PayloadError) -> Self {
        Self::PayloadError(other.to_string())
    }
}

/// An error that can occur while decoding a base64 string into an integer.
#[derive(Debug, Error)]
pub enum IntDecodeError {
    /// An error that occurs if a given string isn't valid base64.
    #[error("{0}")]
    Base64Error(#[from] base64::DecodeError),

    /// An error that occurs if the string was decoded into bytes, but the
    /// bytes can't be properly fit into the desired int size (e.g. 4 bytes).
    #[error("{0}")]
    TryFromSliceError(#[from] TryFromSliceError),
}

/// A struct to make it easier to make database errors to API response errors.
/// By default we assume any DB error to be a server-side problem, and we log
/// and propagate the error. Some DB errors indicate an issue with client input
/// though. In those cases, we need to map the DB error to some other output.
/// This struct encapsulates the most common of these mappings. The main
/// behavior is accessible via [Self::convert].
///
/// Disclaimer: I wrote this in a hurry. It probably has burrs and gaps. Feel
/// free to refactor it later.
#[derive(Copy, Clone, Debug, Default)]
pub struct DbErrorConverter {
    /// Convert DB foreign key violation to [ResponseError::NotFound]? Useful
    /// when inserting or modifying foreign keys. Read the description for
    /// [ResponseError::NotFound] for more info on when this should and
    /// shouldn't be used.
    pub fk_violation_to_not_found: bool,

    /// Convert DB unique violations to [ResponseError::AlreadyExists]? Useful
    /// for insert statements, or updates where all or part of a unique
    /// field can be changed.
    pub unique_violation_to_exists: bool,

    /// Convert [diesel::result::Error::QueryBuilderError] to
    /// [ResponseError::NoUpdate].
    pub query_builder_to_no_update: bool,
}

impl DbErrorConverter {
    /// If the result is an error, convert it from a Diesel error to a
    /// [ResponseError]. If the result is `Ok`, just return it.
    pub fn convert<T: Debug>(
        self,
        result: Result<T, diesel::result::Error>,
    ) -> Result<T, ResponseError> {
        result.map_err(|error| {
            match error {
                // FK is invalid
                diesel::result::Error::DatabaseError(
                    DatabaseErrorKind::ForeignKeyViolation,
                    _,
                ) if self.fk_violation_to_not_found => ClientError::NotFound {
                    source: Some(error),
                }
                .into(),

                // Object already exists
                diesel::result::Error::DatabaseError(
                    DatabaseErrorKind::UniqueViolation,
                    _,
                ) if self.unique_violation_to_exists => {
                    ClientError::AlreadyExists { source: error }.into()
                }

                // User didn't specify any fields to update
                diesel::result::Error::QueryBuilderError(ref msg)
                    if self.query_builder_to_no_update
                    // Currently this is the only way a QueryBuilderError can occur,
                    // but diesel could change that so keep this check to be safe
                        && msg.to_string().contains("no changes to save") =>
                {
                    ClientError::NoUpdate { source: error }.into()
                }

                // Add more conversions here

                // Fall back to the built in converion from ResponseError
                _ => error.into(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use validator::Validate;

    #[derive(Validate)]
    struct TestStructParent {
        #[validate]
        child: TestStructChild,
        #[validate]
        children: Vec<TestStructChild>,
    }

    #[derive(Validate)]
    struct TestStructChild {
        #[validate(range(min = 0))]
        number: i32,
        #[validate(email)]
        email: &'static str,
    }

    #[test]
    fn test_validation_to_field_error() {
        let test_struct = TestStructParent {
            child: TestStructChild {
                number: -1,
                email: "bad-email-1",
            },
            children: vec![
                TestStructChild {
                    number: -2,
                    email: "bad-email-2",
                },
                TestStructChild {
                    number: 0,
                    email: "good@example.com",
                },
                TestStructChild {
                    number: -3,
                    email: "bad-email-3",
                },
            ],
        };
        let server_error: ResponseError =
            test_struct.validate().unwrap_err().into();
        assert_eq!(
            // Juniper's object type has issues with equality checks, so it's
            // easier to convert to JSON, then compare
            json!(server_error.into_field_error().extensions()),
            json!({
                "child": {
                    "number": [{ "min": "0.0", "value": "-1" }],
                    "email": [{ "value": "\"bad-email-1\"" }],
                },
                "children": {
                    "0": {
                        "number": [{ "min": "0.0", "value": "-2" }],
                        "email": [{ "value": "\"bad-email-2\"" }],
                    },
                    "2": {
                        "number": [{ "min": "0.0", "value": "-3" }],
                        "email": [{ "value": "\"bad-email-3\"" }],
                    },
                }
            })
        );
    }

    #[test]
    fn test_db_error_converter_base_case() {
        use diesel::result::Error;
        let converter = DbErrorConverter::default();
        // We need lambdas around these values because we have to move them
        // and diesel's Error doesn't implement Clone
        let make_fk_violation_error = || {
            Error::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                Box::new(String::new()),
            )
        };
        let make_unique_violation_error = || {
            Error::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                Box::new(String::new()),
            )
        };

        // Check all error types with all flags off
        assert_eq!(converter.convert(Ok(3)).unwrap(), 3);
        assert_eq!(
            converter
                .convert::<()>(Err(make_fk_violation_error()))
                .unwrap_err()
                .to_string(),
            ServerError::DieselError(make_fk_violation_error()).to_string()
        );
        assert_eq!(
            converter
                .convert::<()>(Err(make_unique_violation_error()))
                .unwrap_err()
                .to_string(),
            ServerError::DieselError(make_unique_violation_error()).to_string()
        );
    }

    #[test]
    fn test_db_error_converter_fk_violation() {
        assert_eq!(
            DbErrorConverter {
                fk_violation_to_not_found: true,
                ..Default::default()
            }
            .convert::<()>(Err(diesel::result::Error::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                Box::new(String::new()),
            )))
            .unwrap_err()
            .to_string(),
            "Not found"
        );
    }

    #[test]
    fn test_db_error_converter_unique_violation() {
        assert_eq!(
            DbErrorConverter {
                unique_violation_to_exists: true,
                ..Default::default()
            }
            .convert::<()>(Err(diesel::result::Error::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                Box::new(String::new()),
            )))
            .unwrap_err()
            .to_string(),
            "This resource already exists"
        );
    }

    #[test]
    fn test_db_error_converter_query_builder() {
        assert_eq!(
            DbErrorConverter {
                query_builder_to_no_update: true,
                ..Default::default()
            }
            .convert::<()>(Err(diesel::result::Error::QueryBuilderError(
                // If diesel ever changes the message, this will be invalid.
                // We need to rely on API integration tests to catch that
                "There are no changes to save. This query cannot be built"
                    .into(),
            )))
            .unwrap_err()
            .to_string(),
            "No fields were given to update"
        );
    }
}
