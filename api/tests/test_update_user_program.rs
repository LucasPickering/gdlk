#![deny(clippy::all, unused_must_use, unused_imports)]

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

/// Successful row modification
#[test]
fn test_update_user_program_success() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.db_conn();

    // Initialize data
    let user_id = NewUser { username: "user1" }.create(conn).id;
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
        user_id,
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
                "fileName" => InputValue::scalar("new.gdlk"),
                "sourceCode" => InputValue::scalar("WRITE RX0"),
            }
        ),
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

/// Pass an invalid ID, get null back
#[test]
fn test_update_user_program_invalid_id() {
    let runner = QueryRunner::new().unwrap();

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar("bogus".to_string()),
                "fileName" => InputValue::scalar("new.gdlk"),
                "sourceCode" => InputValue::scalar("WRITE RX0"),
            }
        ),
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
#[test]
fn test_update_user_program_empty_modification() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.db_conn();

    // Initialize data
    let user_id = NewUser { username: "user1" }.create(conn).id;
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
        user_id,
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
                "locations": [{"line": 7, "column": 9}],
                "message": "No fields were given to update",
                "path": ["updateUserProgram"],
            })]
        )
    );
}

/// [ERROR] Attempted to use an existing name
#[test]
fn test_update_user_program_duplicate() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.db_conn();

    // Initialize data
    let user_id = NewUser { username: "user1" }.create(conn).id;
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
        user_id,
        program_spec_id,
        file_name: "existing.gdlk",
        source_code: "READ RX0",
    }
    .create(conn);
    // Use this to test collisions
    NewUserProgram {
        user_id,
        program_spec_id,
        file_name: "existing2.gdlk",
        source_code: "READ RX0",
    }
    .create(conn);

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(user_program.id.to_string()),
                "fileName" => InputValue::scalar("existing2.gdlk"),
                "sourceCode" => InputValue::scalar("WRITE RX0"),
            }
        ),
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
#[test]
fn test_update_user_program() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.db_conn();

    // Initialize data
    let user_id = NewUser { username: "user1" }.create(conn).id;
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
        user_id,
        program_spec_id,
        file_name: "existing.gdlk",
        source_code: "READ RX0",
    }
    .create(conn);

    // Error - Known user program, but the target file name is invalid
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(user_program.id.to_string()),
                "fileName" => InputValue::scalar(""),
            }
        ),
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
