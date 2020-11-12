#![deny(clippy::all)]

use crate::utils::{factories::*, QueryRunner};
use diesel_factories::Factory;
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;

mod utils;

static QUERY: &str = r#"
    mutation CopyUserProgramMutation($id: ID!) {
        copyUserProgram(input: {
            id: $id,
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

/// Successful copy
#[actix_rt::test]
async fn test_copy_user_program_success() {
    let mut runner = QueryRunner::new();
    let user = runner.log_in(&[]);

    let user_program = runner.run_with_conn(|conn| {
        // Initialize data
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
            json!({
                "copyUserProgram": {
                    "userProgramEdge": {
                        "node": {
                            "fileName": "existing.gdlk copy",
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

/// Trying to copy a user_program that doesn't exist should return null.
#[actix_rt::test]
async fn test_copy_user_program_invalid_id() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[]);

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar("bogus"),
                }
            )
            .await,
        (
            json!({
                "copyUserProgram": {
                    "userProgramEdge": serde_json::Value::Null
                }
            }),
            vec![]
        )
    );
}

/// Copying while not logged in returns an error.
#[actix_rt::test]
async fn test_copy_user_program_not_logged_in() {
    let runner = QueryRunner::new();

    let user_program = runner.run_with_conn(|conn| {
        // Initialize data
        let user = UserFactory::default().username("user1").insert(&conn);
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
                "locations": [{"line": 3, "column": 9}],
                "message": "Not logged in",
                "path": ["copyUserProgram"],
            })]
        )
    );
}

/// You can't copy user_programs that don't belong to you.
#[actix_rt::test]
async fn test_copy_user_program_wrong_owner() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[]);

    let user_program = runner.run_with_conn(|conn| {
        // Initialize data
        let other_user = UserFactory::default().username("user2").insert(&conn);
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
                }
            )
            .await,
        (
            json!({
                "copyUserProgram": {
                    "userProgramEdge": serde_json::Value::Null
                }
            }),
            vec![]
        )
    );
}
