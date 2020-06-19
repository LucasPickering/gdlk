#![deny(clippy::all)]

use diesel::PgConnection;
use gdlk_api::models::{Factory, NewUser};
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;
use utils::QueryRunner;

mod utils;

#[test]
fn test_field_user() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.db_conn();

    let user_id = NewUser { username: "user1" }.create(conn).id;
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
        runner.query(
            query,
            hashmap! { "username" => InputValue::scalar("user1") }
        ),
        (
            json!({
                "user": {
                    "id": (user_id.to_string()),
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
