#![deny(clippy::all)]

use crate::utils::{factories::*, ContextBuilder, QueryRunner};
use diesel::{OptionalExtension, QueryDsl, RunQueryDsl};
use diesel_factories::Factory;
use gdlk_api::{models, schema::user_programs};
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;

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
    let mut context_builder = ContextBuilder::new();
    let user = context_builder.log_in(&[]);
    let conn = context_builder.db_conn();

    let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(conn);
    let program_spec = ProgramSpecFactory::default()
        .name("prog1")
        .hardware_spec(&hw_spec)
        .insert(conn);
    let user_program = UserProgramFactory::default()
        .user(&user)
        .program_spec(&program_spec)
        .file_name("existing.gdlk")
        .source_code("READ RX0")
        .insert(conn);

    let runner = QueryRunner::new(context_builder);
    // Known row
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(user_program.id.to_string()),
            }
        ),
        (
            json!({
                "deleteUserProgram": {
                    "deletedId": (user_program.id.to_string())
                }
            }),
            vec![]
        )
    );

    // Not in DB anymore
    assert_eq!(
        user_programs::table
            .find(user_program.id)
            .get_result::<models::UserProgram>(runner.db_conn())
            .optional()
            .unwrap(),
        None
    );

    // Deleting again gives a null result
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(user_program.id.to_string()),
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
    let context_builder = ContextBuilder::new();
    let conn = context_builder.db_conn();

    let other_user = UserFactory::default().username("other").insert(conn);
    let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(conn);
    let program_spec = ProgramSpecFactory::default()
        .name("prog1")
        .hardware_spec(&hw_spec)
        .insert(conn);
    let user_program = UserProgramFactory::default()
        .user(&other_user)
        .program_spec(&program_spec)
        .file_name("existing.gdlk")
        .source_code("READ RX0")
        .insert(conn);

    let runner = QueryRunner::new(context_builder);
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
                "path": ["deleteUserProgram"],
            })]
        )
    );
}

/// Test that you can't delete someone else's user_program
#[test]
fn test_delete_user_program_wrong_owner() {
    let mut context_builder = ContextBuilder::new();
    context_builder.log_in(&[]);
    let conn = context_builder.db_conn();

    let other_user = UserFactory::default().username("other").insert(conn);
    let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(conn);
    let program_spec = ProgramSpecFactory::default()
        .name("prog1")
        .hardware_spec(&hw_spec)
        .insert(conn);
    let user_program = UserProgramFactory::default()
        .user(&other_user)
        .program_spec(&program_spec)
        .file_name("existing.gdlk")
        .source_code("READ RX0")
        .insert(conn);

    let runner = QueryRunner::new(context_builder);
    // It should pretend like the user_program doesn't exist
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(user_program.id.to_string()),
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
