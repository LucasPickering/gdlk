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
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.db_conn();

    NewUser { username: "user1" }.create(conn);
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

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "programSpecId" => InputValue::scalar(program_spec_id.to_string()),
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

/// [ERROR] File name is already taken
#[test]
fn test_create_user_program_duplicate() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.db_conn();

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

    // We'll test collisions against this
    NewUserProgram {
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
                "programSpecId" => InputValue::scalar(program_spec_id.to_string()),
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
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.db_conn();

    NewUser { username: "user1" }.create(conn);
    NewProgramSpec {
        name: "prog1",
        hardware_spec_id: NewHardwareSpec {
            name: "hw1",
            ..Default::default()
        }
        .create(conn)
        .id,
        ..Default::default()
    }
    .create(conn);

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
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.db_conn();

    NewUser { username: "user1" }.create(conn);
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

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "programSpecId" => InputValue::scalar(program_spec_id.to_string()),
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
