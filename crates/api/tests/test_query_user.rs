#![deny(clippy::all)]

use crate::utils::{factories::*, QueryRunner};
use diesel_factories::Factory;
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;

mod utils;

#[actix_rt::test]
async fn test_field_user() {
    let runner = QueryRunner::new();

    let user = runner.run_with_conn(|conn| {
        UserFactory::default().username("user1").insert(&conn)
    });

    let query = r#"
        query UserQuery($username: String!) {
            user(username: $username) {
                id
                username
            }
        }
    "#;

    // Known user
    assert_eq!(
        runner
            .query(
                query,
                hashmap! { "username" => InputValue::scalar("user1") }
            )
            .await,
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
        runner
            .query(
                query,
                hashmap! { "username" => InputValue::scalar("unknown_user") }
            )
            .await,
        (json!({ "user": serde_json::Value::Null }), vec![])
    );
}
