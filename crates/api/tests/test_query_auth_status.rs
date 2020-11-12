#![deny(clippy::all)]

use crate::utils::{factories::*, QueryRunner};
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
#[actix_rt::test]
async fn test_field_auth_status_not_logged_in() {
    let runner = QueryRunner::new();

    assert_eq!(
        runner.query(QUERY, hashmap! {}).await,
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
#[actix_rt::test]
async fn test_field_auth_status_user_not_created() {
    let mut runner = QueryRunner::new();

    let user_provider = runner
        .run_with_conn(|conn| UserProviderFactory::default().insert(&conn));
    // user_provider is set, but not user
    runner.set_user_provider(user_provider);

    assert_eq!(
        runner.query(QUERY, hashmap! {}).await,
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
#[actix_rt::test]
async fn test_field_auth_status_user_created() {
    let mut runner = QueryRunner::new();

    let (user_provider, user) = runner.run_with_conn(|conn| {
        let user = UserFactory::default().username("user1").insert(&conn);
        let user_provider = UserProviderFactory::default()
            .user(Some(&user))
            .insert(&conn);
        (user_provider, user)
    });
    runner.set_user_provider(user_provider);

    assert_eq!(
        runner.query(QUERY, hashmap! {}).await,
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
