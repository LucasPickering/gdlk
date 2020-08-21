#![deny(clippy::all)]

use crate::utils::{factories::*, ContextBuilder, QueryRunner};
use diesel_factories::Factory;
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;

mod utils;

static QUERY: &str = r#"
    mutation CreateUserProgramMutation(
        $programSpecId: ID!,
        $fileName: String!,
        $sourceCode: String,
    ) {
        createUserProgram(input: {
            programSpecId: $programSpecId,
            fileName: $fileName,
            sourceCode: $sourceCode,
        }) {
            userProgramEdge {
                node {
                    fileName
                    sourceCode
                    user {
                        username
                    }
                    programSpec {
                        slug
                    }
                }
            }
        }
    }
"#;

/// Test creating a user program successfully
#[test]
fn test_create_user_program_success() {
    let mut context_builder = ContextBuilder::new();
    context_builder.log_in(&[]);
    let conn = context_builder.db_conn();

    let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(conn);
    let program_spec = ProgramSpecFactory::default()
        .name("prog1")
        .hardware_spec(&hw_spec)
        .insert(conn);

    let runner = QueryRunner::new(context_builder);
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "programSpecId" => InputValue::scalar(program_spec.id.to_string()),
                "fileName" => InputValue::scalar("new.gdlk"),
                "sourceCode" => InputValue::scalar("READ RX0"),
            }
        ),
        (
            json!({
                "createUserProgram": {
                    "userProgramEdge": {
                        "node": {
                            "fileName": "new.gdlk",
                            "sourceCode": "READ RX0",
                            "user": {
                                "username": "user1"
                            },
                            "programSpec": {
                                "slug": "prog1"
                            },
                        }
                    }
                }
            }),
            vec![]
        )
    );
}

/// Test that two users can create the a user_program with the same name
#[test]
fn test_create_user_program_repeat_name() {
    let mut context_builder = ContextBuilder::new();
    context_builder.log_in(&[]);
    let conn = context_builder.db_conn();

    let other_user = UserFactory::default().username("other").insert(conn);
    let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(conn);
    let program_spec = ProgramSpecFactory::default()
        .name("prog1")
        .hardware_spec(&hw_spec)
        .insert(conn);
    UserProgramFactory::default()
        .user(&other_user)
        .program_spec(&program_spec)
        .file_name("new.gdlk")
        .insert(conn);

    let runner = QueryRunner::new(context_builder);
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "programSpecId" => InputValue::scalar(program_spec.id.to_string()),
                "fileName" => InputValue::scalar("new.gdlk"),
            }
        ),
        (
            json!({
                "createUserProgram": {
                    "userProgramEdge": {
                        "node": {
                            "fileName": "new.gdlk",
                            "sourceCode": "",
                            "user": {
                                "username": "user1"
                            },
                            "programSpec": {
                                "slug": "prog1"
                            },
                        }
                    }
                }
            }),
            vec![]
        )
    );
}

/// [ERROR] File name is already taken
#[test]
fn test_create_user_program_duplicate() {
    let mut context_builder = ContextBuilder::new();
    let user = context_builder.log_in(&[]);
    let conn = context_builder.db_conn();

    let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(conn);
    let program_spec = ProgramSpecFactory::default()
        .name("prog1")
        .hardware_spec(&hw_spec)
        .insert(conn);

    // We'll test collisions against this
    UserProgramFactory::default()
        .user(&user)
        .program_spec(&program_spec)
        .file_name("existing.gdlk")
        .source_code("READ RX0")
        .insert(conn);

    let runner = QueryRunner::new(context_builder);
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "programSpecId" => InputValue::scalar(program_spec.id.to_string()),
                "fileName" => InputValue::scalar("existing.gdlk"),
            }
        ),
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 7, "column": 9}],
                "message": "This resource already exists",
                "path": ["createUserProgram"],
            })]
        )
    );
}

/// [ERROR] References an invalid program spec
#[test]
fn test_create_user_program_invalid_program_spec() {
    let mut context_builder = ContextBuilder::new();
    context_builder.log_in(&[]);

    let runner = QueryRunner::new(context_builder);
    // Error - Unknown user+program spec combo
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "programSpecId" => InputValue::scalar("garbage"),
                "fileName" => InputValue::scalar("new.gdlk"),
            }
        ),
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 7, "column": 9}],
                "message": "Not found",
                "path": ["createUserProgram"],
            })]
        )
    );
}

/// [ERROR] Values given are invalid
#[test]
fn test_create_user_program_invalid_values() {
    let mut context_builder = ContextBuilder::new();
    context_builder.log_in(&[]);
    let conn = context_builder.db_conn();

    let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(conn);
    let program_spec = ProgramSpecFactory::default()
        .name("prog1")
        .hardware_spec(&hw_spec)
        .insert(conn);

    let runner = QueryRunner::new(context_builder);
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "programSpecId" => InputValue::scalar(program_spec.id.to_string()),
                "fileName" => InputValue::scalar(""),
            }
        ),
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 7, "column": 9}],
                "message": "Input validation error(s)",
                "path": ["createUserProgram"],
                "extensions": {
                    "file_name": [{"min": "1", "value": "\"\""}]
                }
            })]
        )
    );
}
