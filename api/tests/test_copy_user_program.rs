#![deny(clippy::all)]

use diesel::PgConnection;
use gdlk_api::models::{
    Factory, NewHardwareSpec, NewProgramSpec, NewUser, NewUserProgram,
};
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;
use utils::QueryRunner;

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
#[test]
fn test_copy_user_program_success() {
    let mut runner = QueryRunner::new();
    let conn: &PgConnection = &runner.db_conn();

    // Initialize data
    let user = NewUser { username: "user1" }.create(conn);
    let program_spec_id = NewProgramSpec {
        name: "prog1",
        hardware_spec_id: NewHardwareSpec {
            name: "hw1",
            ..Default::default()
        }
        .create(conn)
        .id,
        ..Default::default()
    }
    .create(conn)
    .id;
    let user_program = NewUserProgram {
        user_id: user.id,
        program_spec_id,
        file_name: "existing.gdlk",
        source_code: "READ RX0",
    }
    .create(conn);
    runner.set_user(user); // Log in

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(user_program.id.to_string()),
            }
        ),
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
#[test]
fn test_copy_user_program_invalid_id() {
    let mut runner = QueryRunner::new();
    let conn: &PgConnection = &runner.db_conn();

    // Initialize data
    let user = NewUser { username: "user1" }.create(conn);
    runner.set_user(user); // Log in

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar("bogus"),
            }
        ),
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
#[test]
fn test_copy_user_program_not_logged_in() {
    let runner = QueryRunner::new();
    let conn: &PgConnection = &runner.db_conn();

    // Initialize data
    let user = NewUser { username: "user1" }.create(conn);
    let program_spec_id = NewProgramSpec {
        name: "prog1",
        hardware_spec_id: NewHardwareSpec {
            name: "hw1",
            ..Default::default()
        }
        .create(conn)
        .id,
        ..Default::default()
    }
    .create(conn)
    .id;
    let user_program = NewUserProgram {
        user_id: user.id,
        program_spec_id,
        file_name: "existing.gdlk",
        source_code: "READ RX0",
    }
    .create(conn);

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(user_program.id.to_string()),
            }
        ),
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
#[test]
fn test_copy_user_program_wrong_owner() {
    let mut runner = QueryRunner::new();
    let conn: &PgConnection = &runner.db_conn();

    // Initialize data
    let user = NewUser { username: "user1" }.create(conn);
    let other_user = NewUser { username: "user2" }.create(conn);
    let program_spec_id = NewProgramSpec {
        name: "prog1",
        hardware_spec_id: NewHardwareSpec {
            name: "hw1",
            ..Default::default()
        }
        .create(conn)
        .id,
        ..Default::default()
    }
    .create(conn)
    .id;
    let user_program = NewUserProgram {
        user_id: other_user.id,
        program_spec_id,
        file_name: "existing.gdlk",
        source_code: "READ RX0",
    }
    .create(conn);
    runner.set_user(user); // Log in

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(user_program.id.to_string()),
            }
        ),
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
