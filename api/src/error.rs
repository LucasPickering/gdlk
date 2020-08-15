//! Error types and other error-related code.

use crate::util;
use actix_web::{
    client::{PayloadError, SendRequestError},
    HttpResponse,
};
use diesel::result::DatabaseErrorKind;
use juniper::{DefaultScalarValue, FieldError, IntoFieldError};
use log::error;
use openidconnect::{core::CoreErrorResponseType, StandardErrorResponse};
use std::{array::TryFromSliceError, fmt::Debug};
use thiserror::Error;
use validator::{ValidationError, ValidationErrors, ValidationErrorsKind};

pub type ResponseResult<T> = Result<T, ResponseError>;

/// An error that can occur while handling an HTTP request. These errors should
/// all at least somewhat meaningful to the user.
#[derive(Debug, Error)]
pub enum ResponseError {
    // ===== Client errors =====
    /// User tried to tried to reference a non-existent resource. Be careful
    /// with this! This should NOT be to respond to queries where the missing
    /// resource was directly queried. E.g. if querying hardware specs by slug,
    /// and there is no row with the given slug, the API should return `None`,
    /// NOT this variant! This should be returned when the user implicitly
    /// assumes a resource exists when it does not. For example, insert a new
    /// row and specifying a FK to a related row. If that FK is invalid, that
    /// would be a good time to return this variant.
    #[error("Not found")]
    NotFound,

    /// User tried to use some unique identifier that already exists. This
    /// could occur during a create, rename, etc.
    #[error("This resource already exists")]
    AlreadyExists,

    /// User tried to perform an update mutation, but didn't given any values
    /// to change.
    #[error("No fields were given to update")]
    NoUpdate,

    /// Action cannot be performed because the user is not authenticated.
    #[error("Not logged in")]
    Unauthenticated,

    /// User submitted invalid/incorrect credentials for auth
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// Wrapper for a serde_json error.
    #[error("Serialization/deserialization error: {}", 0)]
    SerializationError(#[from] serde_json::error::Error),

    /// Wrapper for validator's error type
    #[error("Validator error: {}", 0)]
    ValidationErrors(#[from] validator::ValidationErrors),

    /// Wrapper for an OpenID token error, which can occur while validating a
    /// token submitted by a user.
    #[error("{}", 0)]
    RequestTokenError(
        #[from]
        openidconnect::RequestTokenError<
            ActixClientError,
            StandardErrorResponse<CoreErrorResponseType>,
        >,
    ),

    // ===== Server Errors =====
    /// Wrapper for R2D2's error type
    #[error("Database error: {}", 0)]
    R2d2Error(#[from] r2d2::Error),

    /// Wrapper for Diesel's error type
    #[error("Database error: {}", 0)]
    DieselError(#[from] diesel::result::Error),
}

// Juniper error
impl IntoFieldError for ResponseError {
    fn into_field_error(self) -> FieldError {
        match self {
            ResponseError::ValidationErrors(errors) => {
                validation_to_field_error(errors)
            }
            error => FieldError::new(error.to_string(), juniper::Value::Null),
        }
    }
}

// Actix error
impl actix_web::ResponseError for ResponseError {
    fn error_response(&self) -> HttpResponse {
        match self {
            // 401
            Self::InvalidCredentials => HttpResponse::Unauthorized().into(),
            // 409
            Self::AlreadyExists => HttpResponse::Conflict().into(),
            // Everything else becomes a 500
            _ => HttpResponse::InternalServerError().into(),
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
    #[error("{}", 0)]
    SendRequestError(String),

    /// A wrapper for [actix_web::client::PayloadError]. This type doesn't
    /// implement `Error` so we have to convert it to a string first.
    #[error("{}", 0)]
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
    #[error("{}", 0)]
    Base64Error(#[from] base64::DecodeError),

    /// An error that occurs if the string was decoded into bytes, but the
    /// bytes can't be properly fit into the desired int size (e.g. 4 bytes).
    #[error("{}", 0)]
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
        match result {
            Ok(val) => Ok(val),

            // FK is invalid
            Err(diesel::result::Error::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _,
            )) if self.fk_violation_to_not_found => {
                Err(ResponseError::NotFound)
            }

            // Object already exists
            Err(diesel::result::Error::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _,
            )) if self.unique_violation_to_exists => {
                Err(ResponseError::AlreadyExists)
            }

            // User didn't specify any fields to update
            Err(diesel::result::Error::QueryBuilderError(msg))
                if self.query_builder_to_no_update
                // Currently this is the only way a QueryBuilderError can occur,
                // but diesel could change that so keep this check to be safe
                    && msg.to_string().contains("no changes to save") =>
            {
                Err(ResponseError::NoUpdate)
            }

            // Add more conversions here

            // Fall back to the built in converion from ResponseError
            Err(err) => Err(err.into()),
        }
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
            ResponseError::DieselError(make_fk_violation_error()).to_string()
        );
        assert_eq!(
            converter
                .convert::<()>(Err(make_unique_violation_error()))
                .unwrap_err()
                .to_string(),
            ResponseError::DieselError(make_unique_violation_error())
                .to_string()
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
            ResponseError::NotFound.to_string()
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
            ResponseError::AlreadyExists.to_string()
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
            ResponseError::NoUpdate.to_string()
        );
    }
}
