#![deny(clippy::all)]

use diesel::PgConnection;
use gdlk_api::models::{self, Factory, NewHardwareSpec, NewProgramSpec};
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;
use utils::QueryRunner;

mod utils;

#[test]
fn test_program_spec() {
    let runner = QueryRunner::new();
    let conn: &PgConnection = &runner.db_conn();

    let hardware_spec_id = NewHardwareSpec {
        name: "hw1",
        ..Default::default()
    }
    .create(conn)
    .id;
    let program_spec_id = models::NewProgramSpec {
        name: "prog1",
        description: "Program spec!",
        hardware_spec_id,
        input: vec![1, 2, 3],
        expected_output: vec![1, 2, 3],
    }
    .create(conn)
    .id;
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
                        "id": program_spec_id.to_string(),
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
    let mut runner = QueryRunner::new();
    let conn: &PgConnection = &runner.db_conn();

    let user = runner.log_in();
    let hardware_spec_id = NewHardwareSpec {
        name: "hw1",
        ..Default::default()
    }
    .create(conn)
    .id;
    let program_spec_id = NewProgramSpec {
        name: "prog1",
        hardware_spec_id,
        ..Default::default()
    }
    .create(conn)
    .id;
    let user_program_id = models::NewUserProgram {
        user_id: user.id,
        program_spec_id,
        file_name: "sl1.gdlk",
        source_code: "READ RX0",
    }
    .create(conn)
    .id;
    models::NewUserProgram {
        user_id: user.id,
        program_spec_id,
        file_name: "sl2.gdlk",
        source_code: "READ RX0",
    }
    .create(conn);
    models::NewUserProgram {
        user_id: user.id,
        program_spec_id,
        file_name: "sl3.gdlk",
        source_code: "READ RX0",
    }
    .create(conn);
    // Create a new program spec with a new solution for it
    models::NewUserProgram {
        user_id: user.id,
        program_spec_id: NewProgramSpec {
            name: "prog2",
            hardware_spec_id,
            ..Default::default()
        }
        .create(conn)
        .id,
        file_name: "sl1.gdlk",
        source_code: "READ RX0",
    }
    .create(conn);

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
                            "id": (user_program_id.to_string()),
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
    let runner = QueryRunner::new();
    let conn: &PgConnection = &runner.db_conn();

    let hardware_spec_id = NewHardwareSpec {
        name: "hw1",
        ..Default::default()
    }
    .create(conn)
    .id;
    NewProgramSpec {
        name: "prog1",
        hardware_spec_id,
        ..Default::default()
    }
    .create(conn);

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
