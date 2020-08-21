#![deny(clippy::all)]

use crate::utils::{factories::*, ContextBuilder, QueryRunner};
use diesel_factories::Factory;
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;

mod utils;

#[test]
fn test_field_user() {
    let context_builder = ContextBuilder::new();
    let conn = context_builder.db_conn();

    let user = UserFactory::default().username("user1").insert(conn);
    let query = r#"
        query UserQuery($username: String!) {
            user(username: $username) {
                id
                username
            }
        }
    "#;

    let runner = QueryRunner::new(context_builder);
    // Known user
    assert_eq!(
        runner.query(
            query,
            hashmap! { "username" => InputValue::scalar("user1") }
        ),
        (
            json!({
                "user": {
                    "id": (user.id.to_string()),
                    "username": "user1"
                }
            }),
            vec![]
        )
    );

    // Unknown user
    assert_eq!(
        runner.query(
            query,
            hashmap! { "username" => InputValue::scalar("unknown_user") }
        ),
        (json!({ "user": serde_json::Value::Null }), vec![])
    );
}
