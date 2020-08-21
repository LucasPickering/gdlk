#![deny(clippy::all)]

use crate::utils::{factories::*, ContextBuilder, QueryRunner};
use diesel_factories::Factory;
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;

mod utils;

#[test]
fn test_program_spec() {
    let context_builder = ContextBuilder::new();
    let conn = context_builder.db_conn();

    let hardware_spec = HardwareSpecFactory::default().name("hw1").insert(conn);
    let program_spec = ProgramSpecFactory::default()
        .name("prog1")
        .hardware_spec(&hardware_spec)
        .input(vec![1, 2, 3])
        .expected_output(vec![1, 2, 3])
        .insert(conn);

    let query = r#"
        query ProgramSpecQuery($hwSlug: String!, $progSlug: String!) {
            hardwareSpec(slug: $hwSlug) {
                programSpec(slug: $progSlug) {
                    id
                    slug
                    input
                    expectedOutput
                    hardwareSpec {
                        slug
                    }
                }
            }
        }
    "#;

    let runner = QueryRunner::new(context_builder);
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "hwSlug" => InputValue::scalar("hw1"),
                "progSlug" => InputValue::scalar("prog1"),
            }
        ),
        (
            json!({
                "hardwareSpec": {
                    "programSpec": {
                        "id": program_spec.id.to_string(),
                        "slug": "prog1",
                        "input": vec![1, 2, 3],
                        "expectedOutput": vec![1, 2, 3],
                        "hardwareSpec": {
                            "slug": "hw1"
                        }
                    }
                }
            }),
            vec![]
        )
    );
}

#[test]
fn test_program_spec_user_program() {
    let mut context_builder = ContextBuilder::new();
    let user = context_builder.log_in(&[]);
    let conn = context_builder.db_conn();

    let hardware_spec = HardwareSpecFactory::default().name("hw1").insert(conn);
    let program_spec = ProgramSpecFactory::default()
        .name("prog1")
        .hardware_spec(&hardware_spec)
        .insert(conn);

    let user_program = UserProgramFactory::default()
        .user(&user)
        .program_spec(&program_spec)
        .file_name("sl1.gdlk")
        .source_code("READ RX0")
        .insert(conn);
    UserProgramFactory::default()
        .user(&user)
        .program_spec(&program_spec)
        .file_name("sl2.gdlk")
        .insert(conn);
    UserProgramFactory::default()
        .user(&user)
        .program_spec(&program_spec)
        .file_name("sl3.gdlk")
        .insert(conn);

    // Create a new program spec with a new solution for it
    let program_spec2 = ProgramSpecFactory::default()
        .name("prog2")
        .hardware_spec(&hardware_spec)
        .insert(conn);
    UserProgramFactory::default()
        .user(&user)
        .program_spec(&program_spec2)
        .file_name("sl1.gdlk")
        .insert(conn);

    let runner = QueryRunner::new(context_builder);
    let query = r#"
        query UserProgramQuery(
            $hwSlug: String!,
            $progSlug: String!,
            $fileName: String!,
        ) {
            hardwareSpec(slug: $hwSlug) {
                programSpec(slug: $progSlug) {
                    userProgram(fileName: $fileName) {
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
                    userPrograms {
                        totalCount
                        edges {
                            node {
                                fileName
                            }
                        }
                    }
                }
            }
        }
    "#;

    // Query a set of user programs
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "hwSlug" => InputValue::scalar("hw1"),
                "progSlug" => InputValue::scalar("prog1"),
                "fileName" => InputValue::scalar("sl1.gdlk"),
            }
        ),
        (
            json!({
                "hardwareSpec": {
                    "programSpec": {
                        "userProgram": {
                            "id": (user_program.id.to_string()),
                            "fileName": "sl1.gdlk",
                            "sourceCode": "READ RX0",
                            "user": {
                                "username": "user1"
                            },
                            "programSpec": {
                                "slug": "prog1"
                            },
                        },
                        "userPrograms": {
                            "totalCount": 3,
                            "edges": [
                                {
                                    "node": {
                                        "fileName": "sl1.gdlk"
                                    }
                                },
                                {
                                    "node": {
                                        "fileName": "sl2.gdlk"
                                    }
                                },
                                {
                                    "node": {
                                        "fileName": "sl3.gdlk"
                                    }
                                },
                            ]
                        }
                    }
                }
            }),
            vec![]
        )
    );
}

/// Test that querying anything user_program related while not logged in
/// triggers an error.
#[test]
fn test_program_spec_user_program_not_logged_in() {
    let context_builder = ContextBuilder::new();
    let conn = context_builder.db_conn();

    let hardware_spec = HardwareSpecFactory::default().name("hw1").insert(conn);
    ProgramSpecFactory::default()
        .name("prog1")
        .hardware_spec(&hardware_spec)
        .insert(conn);

    let runner = QueryRunner::new(context_builder);
    let query = r#"
        query UserProgramQuery(
            $hwSlug: String!,
            $progSlug: String!,
            $fileName: String!,
        ) {
            hardwareSpec(slug: $hwSlug) {
                programSpec(slug: $progSlug) {
                    userProgram(fileName: $fileName) {
                        id
                    }
                    userPrograms {
                        totalCount
                    }
                }
            }
        }
    "#;

    // Query a set of user programs
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "hwSlug" => InputValue::scalar("hw1"),
                "progSlug" => InputValue::scalar("prog1"),
                "fileName" => InputValue::scalar("sl1.gdlk"),
            }
        ),
        (
            json!({
                "hardwareSpec": {
                    "programSpec": serde_json::Value::Null
                }
            }),
            vec![
                json!({
                    "message": "Not logged in",
                    "locations": [{"line": 9, "column": 21}],
                    "path": ["hardwareSpec", "programSpec", "userProgram"],
                }),
                json!({
                    "message": "Not logged in",
                    "locations": [{"line": 12, "column": 21}],
                    "path": ["hardwareSpec", "programSpec", "userPrograms"],
                }),
            ]
        )
    );
}
