//! Error types and other error-related code.

use crate::util;
use actix_web::{HttpResponse, ResponseError};
use failure::Fail;
use juniper::{DefaultScalarValue, FieldError, IntoFieldError};
use validator::{ValidationError, ValidationErrors, ValidationErrorsKind};

pub type ServerResult<T> = Result<T, ServerError>;

#[derive(Debug, Fail)]
pub enum ServerError {
    /// Wrapper for R2D2's error type
    #[fail(display = "{}", 0)]
    R2d2Error(#[cause] r2d2::Error),

    /// Wrapper for Diesel's error type
    #[fail(display = "{}", 0)]
    DieselError(#[cause] diesel::result::Error),

    /// Wrapper for validator's error type
    #[fail(display = "{}", 0)]
    ValidationErrors(#[cause] validator::ValidationErrors),

    /// Wrapper for uuid's parse error type
    #[fail(display = "{}", 0)]
    UuidParseError(#[cause] uuid::parser::ParseError),
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

impl From<ValidationErrors> for ServerError {
    fn from(other: ValidationErrors) -> Self {
        Self::ValidationErrors(other)
    }
}

impl From<uuid::parser::ParseError> for ServerError {
    fn from(other: uuid::parser::ParseError) -> Self {
        Self::UuidParseError(other)
    }
}

// Juniper error
impl IntoFieldError for ServerError {
    fn into_field_error(self) -> FieldError {
        match self {
            ServerError::ValidationErrors(errors) => {
                validation_to_field_error(errors)
            }
            ServerError::UuidParseError(error) => FieldError::new(
                "Error decoding UUID",
                juniper::Value::Scalar(juniper::DefaultScalarValue::String(
                    error.to_string(),
                )),
            ),
            error => FieldError::new(error.to_string(), juniper::Value::Null),
        }
    }
}

// Actix error
impl ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse {
        match self {
            // 404
            Self::DieselError(diesel::result::Error::NotFound) => {
                HttpResponse::NotFound().into()
            }
            // 500
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use uuid::Uuid;
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
        let server_error: ServerError =
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
    fn test_uuid_to_field_error() {
        let server_error: ServerError =
            Uuid::parse_str("bad-uuid").unwrap_err().into();
        assert_eq!(
            server_error.into_field_error(),
            FieldError::new(
                "Error decoding UUID",
                juniper::Value::Scalar(juniper::DefaultScalarValue::String(
                    "invalid length: expected one of [36, 32], found 8".into()
                ))
            )
        );
    }
}
