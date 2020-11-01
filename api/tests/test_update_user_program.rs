#![deny(clippy::all)]

use crate::utils::{factories::*, QueryRunner};
use diesel_factories::Factory;
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;

mod utils;

static QUERY: &str = r#"
    mutation UpdateUserProgramMutation(
        $id: ID!,
        $fileName: String,
        $sourceCode: String,
    ) {
        updateUserProgram(input: {
            id: $id,
            fileName: $fileName,
            sourceCode: $sourceCode,
        }) {
            userProgramEdge {
                node {
                    id
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

static QUERY_MIN: &str = r#"
    mutation UpdateUserProgramMutation(
        $id: ID!,
        $fileName: String,
        $sourceCode: String,
    ) {
        updateUserProgram(input: {
            id: $id,
            fileName: $fileName,
            sourceCode: $sourceCode,
        }) {
            userProgramEdge {
                node {
                    id
                    record {
                        id
                    }
                }
            }
        }
    }
"#;

/// Successful row modification
#[actix_rt::test]
async fn test_update_user_program_success() {
    let mut runner = QueryRunner::new();
    let user = runner.log_in(&[]);

    // Initialize data
    let user_program = runner.run_with_conn(|conn| {
        let hardware_spec =
            HardwareSpecFactory::default().name("hw1").insert(&conn);
        let program_spec = ProgramSpecFactory::default()
            .name("prog1")
            .hardware_spec(&hardware_spec)
            .insert(&conn);
        UserProgramFactory::default()
            .user(&user)
            .program_spec(&program_spec)
            .file_name("existing.gdlk")
            .source_code("READ RX0")
            .insert(&conn)
    });

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar(user_program.id.to_string()),
                    "fileName" => InputValue::scalar("new.gdlk"),
                    "sourceCode" => InputValue::scalar("WRITE RX0"),
                }
            )
            .await,
        (
            json!({
                "updateUserProgram": {
                    "userProgramEdge": {
                        "node": {
                            "id": (user_program.id.to_string()),
                            "fileName": "new.gdlk",
                            "sourceCode": "WRITE RX0",
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

/// Test that the `record_id` column is cleared whenever we modify source, but
/// NOT when other fields are modified.
#[actix_rt::test]
async fn test_update_user_program_clear_record() {
    let mut runner = QueryRunner::new();
    let user = runner.log_in(&[]);

    // Initialize data
    let (user_program, record) = runner.run_with_conn(|conn| {
        let hardware_spec =
            HardwareSpecFactory::default().name("hw1").insert(&conn);
        let program_spec = ProgramSpecFactory::default()
            .name("prog1")
            .hardware_spec(&hardware_spec)
            .insert(&conn);
        let record = UserProgramRecordFactory::default()
            .user(&user)
            .program_spec(&program_spec)
            .insert(&conn);
        let user_program = UserProgramFactory::default()
            .user(&user)
            .program_spec(&program_spec)
            .record(Some(&record))
            .file_name("existing.gdlk")
            .source_code("READ RX0")
            .insert(&conn);
        (user_program, record)
    });

    assert_eq!(
        runner
            .query(
                QUERY_MIN,
                hashmap! {
                    "id" => InputValue::scalar(user_program.id.to_string()),
                    "fileName" => InputValue::scalar("new.gdlk"),
                }
            )
            .await,
        (
            json!({
                "updateUserProgram": {
                    "userProgramEdge": {
                        "node": {
                            "id": (user_program.id.to_string()),
                            "record": {
                                "id": (record.id.to_string())
                            }
                        }
                    }
                }
            }),
            vec![]
        )
    );

    // Modify source, should wipe out record_id
    assert_eq!(
        runner
            .query(
                QUERY_MIN,
                hashmap! {
                    "id" => InputValue::scalar(user_program.id.to_string()),
                    "sourceCode" => InputValue::scalar("WRITE RX0"),
                }
            )
            .await,
        (
            json!({
                "updateUserProgram": {
                    "userProgramEdge": {
                        "node": {
                            "id": (user_program.id.to_string()),
                            "record": serde_json::Value::Null
                        }
                    }
                }
            }),
            vec![]
        )
    );
}

/// No user logged in, gives an auth error
#[actix_rt::test]
async fn test_update_user_program_not_logged_in() {
    let runner = QueryRunner::new();

    // Initialize data
    let user_program = runner.run_with_conn(|conn| {
        let other_user = UserFactory::default().username("other").insert(&conn);
        let hardware_spec =
            HardwareSpecFactory::default().name("hw1").insert(&conn);
        let program_spec = ProgramSpecFactory::default()
            .name("prog1")
            .hardware_spec(&hardware_spec)
            .insert(&conn);
        UserProgramFactory::default()
            .user(&other_user)
            .program_spec(&program_spec)
            .file_name("existing.gdlk")
            .source_code("READ RX0")
            .insert(&conn)
    });

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar(user_program.id.to_string()),
                    "fileName" => InputValue::scalar("new.gdlk"),
                    "sourceCode" => InputValue::scalar("WRITE RX0"),
                }
            )
            .await,
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 7, "column": 9}],
                "message": "Not logged in",
                "path": ["updateUserProgram"],
            })]
        )
    );
}

/// Try to modify someone else's program, it should behave like it doesn't exist
#[actix_rt::test]
async fn test_update_user_program_wrong_owner() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[]);

    // Initialize data
    let user_program = runner.run_with_conn(|conn| {
        let other_user = UserFactory::default().username("other").insert(&conn);
        let hardware_spec =
            HardwareSpecFactory::default().name("hw1").insert(&conn);
        let program_spec = ProgramSpecFactory::default()
            .name("prog1")
            .hardware_spec(&hardware_spec)
            .insert(&conn);
        UserProgramFactory::default()
            .user(&other_user)
            .program_spec(&program_spec)
            .file_name("existing.gdlk")
            .source_code("READ RX0")
            .insert(&conn)
    });

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar(user_program.id.to_string()),
                    "fileName" => InputValue::scalar("new.gdlk"),
                    "sourceCode" => InputValue::scalar("WRITE RX0"),
                }
            )
            .await,
        (
            json!({
                "updateUserProgram": {
                    "userProgramEdge": serde_json::Value::Null
                }
            }),
            vec![]
        )
    );
}

/// [ERROR] No fields were updated
#[actix_rt::test]
async fn test_update_user_program_empty_modification() {
    let mut runner = QueryRunner::new();
    let user = runner.log_in(&[]);

    // Initialize data
    let user_program = runner.run_with_conn(|conn| {
        let hardware_spec =
            HardwareSpecFactory::default().name("hw1").insert(&conn);
        let program_spec = ProgramSpecFactory::default()
            .name("prog1")
            .hardware_spec(&hardware_spec)
            .insert(&conn);
        UserProgramFactory::default()
            .user(&user)
            .program_spec(&program_spec)
            .file_name("existing.gdlk")
            .source_code("READ RX0")
            .insert(&conn)
    });

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar(user_program.id.to_string()),
                }
            )
            .await,
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 7, "column": 9}],
                "message": "No fields were given to update",
                "path": ["updateUserProgram"],
            })]
        )
    );
}

/// [ERROR] Attempted to use an existing name
#[actix_rt::test]
async fn test_update_user_program_duplicate() {
    let mut runner = QueryRunner::new();
    let user = runner.log_in(&[]);

    // Initialize data
    let user_program = runner.run_with_conn(|conn| {
        let hardware_spec =
            HardwareSpecFactory::default().name("hw1").insert(&conn);
        let program_spec = ProgramSpecFactory::default()
            .name("prog1")
            .hardware_spec(&hardware_spec)
            .insert(&conn);
        // Use this to test collisions
        UserProgramFactory::default()
            .user(&user)
            .program_spec(&program_spec)
            .file_name("existing2.gdlk")
            .source_code("READ RX0")
            .insert(&conn);
        // This is the one we'll be editing
        UserProgramFactory::default()
            .user(&user)
            .program_spec(&program_spec)
            .file_name("existing.gdlk")
            .source_code("READ RX0")
            .insert(&conn)
    });

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar(user_program.id.to_string()),
                    "fileName" => InputValue::scalar("existing2.gdlk"),
                    "sourceCode" => InputValue::scalar("WRITE RX0"),
                }
            )
            .await,
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 7, "column": 9}],
                "message": "This resource already exists",
                "path": ["updateUserProgram"],
            })]
        )
    );
}

/// [ERROR] Invalid values passed
#[actix_rt::test]
async fn test_update_user_program_invalid_values() {
    let mut runner = QueryRunner::new();
    let user = runner.log_in(&[]);

    // Initialize data
    let user_program = runner.run_with_conn(|conn| {
        let hardware_spec =
            HardwareSpecFactory::default().name("hw1").insert(&conn);
        let program_spec = ProgramSpecFactory::default()
            .name("prog1")
            .hardware_spec(&hardware_spec)
            .insert(&conn);
        UserProgramFactory::default()
            .user(&user)
            .program_spec(&program_spec)
            .file_name("existing.gdlk")
            .source_code("READ RX0")
            .insert(&conn)
    });

    // Error - Known user program, but the target file name is invalid
    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar(user_program.id.to_string()),
                    "fileName" => InputValue::scalar(""),
                }
            )
            .await,
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 7, "column": 9}],
                "message": "Input validation error(s)",
                "path": ["updateUserProgram"],
                "extensions": {
                    "file_name": [{"min": "1", "value": "\"\""}]
                }
            })]
        )
    );
}
