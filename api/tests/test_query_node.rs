#![deny(clippy::all)]

use diesel::PgConnection;
use gdlk_api::models::{
    self, Factory, NewHardwareSpec, NewProgramSpec, NewUser,
};
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;
use utils::QueryRunner;

mod utils;

#[test]
fn test_field_node() {
    let runner = QueryRunner::new();
    let conn: &PgConnection = &runner.db_conn();

    let user_id = NewUser { username: "user1" }.create(conn).id;
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
        user_id,
        program_spec_id,
        file_name: "prog.gdlk",
        source_code: "",
    }
    .create(conn)
    .id;

    let query = r#"
        query NodeQuery($nodeId: ID!) {
            node(id: $nodeId) {
                id
            }
        }
    "#;

    // Check a known UUID for each model type
    for id in &[user_id, hardware_spec_id, program_spec_id, user_program_id] {
        assert_eq!(
            runner.query(
                query,
                hashmap! {
                    "nodeId" => InputValue::scalar(id.to_string()),
                }
            ),
            (
                json!({
                    "node": {
                        "id": id.to_string(),
                    }
                }),
                vec![]
            )
        );
    }

    // Check an invalid UUID, and a valid UUID that isn't in the DB
    for id in &["invalid-uuid", "1bb9a3a1-0149-4264-a0a7-ff17ac7b8a61"] {
        assert_eq!(
            runner.query(
                query,
                hashmap! {
                    "nodeId" => InputValue::scalar(*id),
                }
            ),
            (
                json!({
                    "node": serde_json::Value::Null,
                }),
                vec![]
            )
        );
    }
}
