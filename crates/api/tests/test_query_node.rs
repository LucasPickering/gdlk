#![deny(clippy::all)]

use crate::utils::{factories::*, QueryRunner};
use diesel_factories::Factory;
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;

mod utils;

#[actix_rt::test]
async fn test_field_node() {
    let runner = QueryRunner::new();

    let node_ids = runner.run_with_conn(|conn| {
        let user = UserFactory::default().username("user1").insert(&conn);
        let hardware_spec =
            HardwareSpecFactory::default().name("HW 1").insert(&conn);
        let program_spec = ProgramSpecFactory::default()
            .name("prog1")
            .hardware_spec(&hardware_spec)
            .insert(&conn);
        let user_program = UserProgramFactory::default()
            .user(&user)
            .program_spec(&program_spec)
            .file_name("new.gdlk")
            .insert(&conn);
        [user.id, hardware_spec.id, program_spec.id, user_program.id]
    });

    let query = r#"
    query NodeQuery($nodeId: ID!) {
        node(id: $nodeId) {
            id
        }
    }
    "#;

    // Check a known UUID for each model type
    for id in &node_ids {
        assert_eq!(
            runner
                .query(
                    query,
                    hashmap! {
                        "nodeId" => InputValue::scalar(id.to_string()),
                    }
                )
                .await,
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
            runner
                .query(
                    query,
                    hashmap! {
                        "nodeId" => InputValue::scalar(*id),
                    }
                )
                .await,
            (
                json!({
                    "node": serde_json::Value::Null,
                }),
                vec![]
            )
        );
    }
}
