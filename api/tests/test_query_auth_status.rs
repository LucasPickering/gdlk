#![deny(clippy::all)]

use diesel::PgConnection;
use gdlk_api::models::{Factory, NewUser, NewUserProvider};
use maplit::hashmap;
use serde_json::json;
use utils::QueryRunner;

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
    let runner = QueryRunner::new();

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
    let mut runner = QueryRunner::new();
    let conn: &PgConnection = &runner.db_conn();
    let user_provider = NewUserProvider {
        sub: "asdf",
        provider_name: "provider",
        user_id: None,
    }
    .create(conn);
    // user_provider is set, but not user
    runner.set_user_provider(user_provider);

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
    let mut runner = QueryRunner::new();
    let conn: &PgConnection = &runner.db_conn();
    let user = NewUser { username: "user1" }.create(conn);
    let user_provider = NewUserProvider {
        sub: "asdf",
        provider_name: "provider",
        user_id: Some(user.id),
    }
    .create(conn);
    // user_provider is set, but not user
    runner.set_user_provider(user_provider);

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
