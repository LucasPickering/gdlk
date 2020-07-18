#![deny(clippy::all)]

use diesel::{OptionalExtension, PgConnection, QueryDsl, RunQueryDsl};
use gdlk_api::{
    models::{
        self, Factory, NewHardwareSpec, NewProgramSpec, NewUser, NewUserProgram,
    },
    schema::user_programs,
};
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;
use utils::QueryRunner;

mod utils;

static QUERY: &str = r#"
    mutation DeleteUserProgramMutation($id: ID!) {
        deleteUserProgram(input: { id: $id }) {
            deletedId
        }
    }
"#;

#[test]
fn test_delete_user_program_success() {
    let mut runner = QueryRunner::new();
    let conn: &PgConnection = &runner.db_conn();

    let user = NewUser { username: "user1" }.create(conn);
    let user_program_id = NewUserProgram {
        user_id: user.id,
        program_spec_id: NewProgramSpec {
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
        .id,
        file_name: "existing.gdlk",
        source_code: "READ RX0",
    }
    .create(conn)
    .id;
    runner.set_user(user); // Log in

    // Known row
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(user_program_id.to_string()),
            }
        ),
        (
            json!({
                "deleteUserProgram": {
                    "deletedId": (user_program_id.to_string())
                }
            }),
            vec![]
        )
    );

    // Not in DB anymore
    assert_eq!(
        user_programs::table
            .find(user_program_id)
            .get_result::<models::UserProgram>(conn)
            .optional()
            .unwrap(),
        None
    );

    // Deleting again gives a null result
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(user_program_id.to_string()),
            }
        ),
        (
            json!({
                "deleteUserProgram": {
                    "deletedId": serde_json::Value::Null
                }
            }),
            vec![]
        )
    );
}

#[test]
fn test_delete_user_program_not_logged_in() {
    let runner = QueryRunner::new();
    let conn: &PgConnection = &runner.db_conn();

    let user_program_id = NewUserProgram {
        user_id: NewUser { username: "user1" }.create(conn).id,
        program_spec_id: NewProgramSpec {
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
        .id,
        file_name: "existing.gdlk",
        source_code: "READ RX0",
    }
    .create(conn)
    .id;

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(user_program_id.to_string()),
            }
        ),
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 3, "column": 9}],
                "message": "Not logged in",
                "path": ["deleteUserProgram"],
            })]
        )
    );
}

/// Test that you can't delete someone else's user_program
#[test]
fn test_delete_user_program_wrong_owner() {
    let mut runner = QueryRunner::new();
    let conn: &PgConnection = &runner.db_conn();

    let user_program_id = NewUserProgram {
        user_id: NewUser { username: "user1" }.create(conn).id,
        program_spec_id: NewProgramSpec {
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
        .id,
        file_name: "existing.gdlk",
        source_code: "READ RX0",
    }
    .create(conn)
    .id;
    let not_owner = NewUser { username: "user2" }.create(conn);
    runner.set_user(not_owner); // Log in as someone else

    // It should pretend like the user_program doesn't exist
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(user_program_id.to_string()),
            }
        ),
        (
            json!({
                "deleteUserProgram": {
                    "deletedId": serde_json::Value::Null
                }
            }),
            vec![]
        )
    );
}
