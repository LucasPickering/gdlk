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
        deleteUserProgram(input: { userProgramId: $id }) {
            deletedId
        }
    }
"#;

#[test]
fn test_delete_user_program() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.db_conn();

    let user_program_id = NewUserProgram {
        user_id: NewUser {
            username: "user1",
            ..Default::default()
        }
        .create(conn)
        .id,
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
