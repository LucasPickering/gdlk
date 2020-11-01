#![deny(clippy::all)]

use crate::utils::{factories::*, QueryRunner};
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
#[actix_rt::test]
async fn test_create_user_program_success() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[]);

    let program_spec = runner.run_with_conn(|conn| {
        let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(&conn);
        ProgramSpecFactory::default()
            .name("prog1")
            .hardware_spec(&hw_spec)
            .insert(&conn)
    });

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "programSpecId" => InputValue::scalar(program_spec.id.to_string()),
                "fileName" => InputValue::scalar("new.gdlk"),
                "sourceCode" => InputValue::scalar("READ RX0"),
            }
        ).await,
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
#[actix_rt::test]
async fn test_create_user_program_repeat_name() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[]);

    let program_spec = runner.run_with_conn(|conn| {
        let other_user = UserFactory::default().username("other").insert(&conn);
        let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(&conn);
        let program_spec = ProgramSpecFactory::default()
            .name("prog1")
            .hardware_spec(&hw_spec)
            .insert(&conn);
        UserProgramFactory::default()
            .user(&other_user)
            .program_spec(&program_spec)
            .file_name("new.gdlk")
            .insert(&conn);
        program_spec
    });

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "programSpecId" => InputValue::scalar(program_spec.id.to_string()),
                "fileName" => InputValue::scalar("new.gdlk"),
            }
        ).await,
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
#[actix_rt::test]
async fn test_create_user_program_duplicate() {
    let mut runner = QueryRunner::new();
    let user = runner.log_in(&[]);

    let program_spec = runner.run_with_conn(|conn| {
        let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(&conn);
        let program_spec = ProgramSpecFactory::default()
            .name("prog1")
            .hardware_spec(&hw_spec)
            .insert(&conn);

        // We'll test collisions against this
        UserProgramFactory::default()
            .user(&user)
            .program_spec(&program_spec)
            .file_name("existing.gdlk")
            .source_code("READ RX0")
            .insert(&conn);
        program_spec
    });

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "programSpecId" => InputValue::scalar(program_spec.id.to_string()),
                "fileName" => InputValue::scalar("existing.gdlk"),
            }
        ).await,
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
#[actix_rt::test]
async fn test_create_user_program_invalid_program_spec() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[]);

    // Error - Unknown user+program spec combo
    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "programSpecId" => InputValue::scalar("garbage"),
                    "fileName" => InputValue::scalar("new.gdlk"),
                }
            )
            .await,
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
#[actix_rt::test]
async fn test_create_user_program_invalid_values() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[]);

    let program_spec = runner.run_with_conn(|conn| {
        let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(&conn);
        ProgramSpecFactory::default()
            .name("prog1")
            .hardware_spec(&hw_spec)
            .insert(&conn)
    });

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "programSpecId" => InputValue::scalar(program_spec.id.to_string()),
                "fileName" => InputValue::scalar(""),
            }
        ).await,
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
