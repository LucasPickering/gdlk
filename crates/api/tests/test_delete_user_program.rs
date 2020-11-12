#![deny(clippy::all)]

use crate::utils::{factories::*, QueryRunner};
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

#[actix_rt::test]
async fn test_delete_user_program_success() {
    let mut runner = QueryRunner::new();
    let user = runner.log_in(&[]);

    let user_program = runner.run_with_conn(|conn| {
        let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(&conn);
        let program_spec = ProgramSpecFactory::default()
            .name("prog1")
            .hardware_spec(&hw_spec)
            .insert(&conn);
        UserProgramFactory::default()
            .user(&user)
            .program_spec(&program_spec)
            .file_name("existing.gdlk")
            .source_code("READ RX0")
            .insert(&conn)
    });

    // Known row
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
                "deleteUserProgram": {
                    "deletedId": (user_program.id.to_string())
                }
            }),
            vec![]
        )
    );

    // Not in DB anymore
    runner.run_with_conn(|conn| {
        assert_eq!(
            user_programs::table
                .find(user_program.id)
                .get_result::<models::UserProgram>(conn)
                .optional()
                .unwrap(),
            None
        );
    });

    // Deleting again gives a null result
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
                "deleteUserProgram": {
                    "deletedId": serde_json::Value::Null
                }
            }),
            vec![]
        )
    );
}

#[actix_rt::test]
async fn test_delete_user_program_not_logged_in() {
    let runner = QueryRunner::new();

    let user_program = runner.run_with_conn(|conn| {
        let other_user = UserFactory::default().username("other").insert(&conn);
        let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(&conn);
        let program_spec = ProgramSpecFactory::default()
            .name("prog1")
            .hardware_spec(&hw_spec)
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
#[actix_rt::test]
async fn test_delete_user_program_wrong_owner() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[]);

    let user_program = runner.run_with_conn(|conn| {
        let other_user = UserFactory::default().username("other").insert(&conn);
        let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(&conn);
        let program_spec = ProgramSpecFactory::default()
            .name("prog1")
            .hardware_spec(&hw_spec)
            .insert(&conn);
        UserProgramFactory::default()
            .user(&other_user)
            .program_spec(&program_spec)
            .file_name("existing.gdlk")
            .source_code("READ RX0")
            .insert(&conn)
    });

    // It should pretend like the user_program doesn't exist
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
                "deleteUserProgram": {
                    "deletedId": serde_json::Value::Null
                }
            }),
            vec![]
        )
    );
}
