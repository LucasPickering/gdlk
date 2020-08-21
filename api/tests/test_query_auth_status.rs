#![deny(clippy::all)]

use crate::utils::{factories::*, ContextBuilder, QueryRunner};
use diesel_factories::Factory;
use maplit::hashmap;
use serde_json::json;

mod utils;

static QUERY: &str = r#"
query AuthStatusQuery {
    authStatus {
        authenticated
        userCreated
        user {
            id
            username
        }
    }
}
"#;

/// Test when the user isn't authenticated at all
#[test]
fn test_field_auth_status_not_logged_in() {
    let context_builder = ContextBuilder::new();
    let runner = QueryRunner::new(context_builder);

    assert_eq!(
        runner.query(QUERY, hashmap! {}),
        (
            json!({
                "authStatus": {
                    "authenticated": false,
                    "userCreated": false,
                    "user": serde_json::Value::Null,
                }
            }),
            vec![]
        )
    );
}

/// Test when the user is logged in, but hasn't finished user setup yet
#[test]
fn test_field_auth_status_user_not_created() {
    let mut context_builder = ContextBuilder::new();
    let conn = context_builder.db_conn();

    let user_provider = UserProviderFactory::default().insert(conn);
    // user_provider is set, but not user
    context_builder.set_user_provider(user_provider);

    let runner = QueryRunner::new(context_builder);
    assert_eq!(
        runner.query(QUERY, hashmap! {}),
        (
            json!({
                "authStatus": {
                    "authenticated": true,
                    "userCreated": false,
                    "user": serde_json::Value::Null,
                }
            }),
            vec![]
        )
    );
}

/// Test when the user is logged in and has created their user. This should be
/// the most common auth status.
#[test]
fn test_field_auth_status_user_created() {
    let mut context_builder = ContextBuilder::new();
    let conn = context_builder.db_conn();

    let user = UserFactory::default().username("user1").insert(conn);
    let user_provider = UserProviderFactory::default()
        .user(Some(&user))
        .insert(conn);
    // user_provider is set, but not user
    context_builder.set_user_provider(user_provider);

    let runner = QueryRunner::new(context_builder);
    assert_eq!(
        runner.query(QUERY, hashmap! {}),
        (
            json!({
                "authStatus": {
                    "authenticated": true,
                    "userCreated": true,
                    "user": {
                        "id": user.id.to_string(),
                        "username": "user1",
                    },
                }
            }),
            vec![]
        )
    );
}
