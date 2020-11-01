#![deny(clippy::all)]

use crate::utils::{factories::*, QueryRunner};
use diesel::{dsl, QueryDsl, RunQueryDsl};
use diesel_factories::Factory;
use gdlk_api::schema::users;
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;

mod utils;

static QUERY: &str = r#"
    mutation InitializeUserMutation(
        $username: String!,
    ) {
        initializeUser(input: {
            username: $username
        }) {
            userEdge {
                node {
                    username
                }
            }
        }
    }
"#;

/// Initialize a user successfully
#[actix_rt::test]
async fn test_initialize_user_success() {
    let mut runner = QueryRunner::new();
    runner.disable_transaction();

    let user_provider = runner
        .run_with_conn(|conn| UserProviderFactory::default().insert(&conn));
    runner.set_user_provider(user_provider);

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "username" => InputValue::scalar("user1"),
                }
            )
            .await,
        (
            json!({
                "initializeUser": {
                    "userEdge": {
                        "node": {
                            "username": "user1"
                        }
                    }
                }
            }),
            vec![]
        )
    );
}

/// Try to initialize a user while not logged in.
#[actix_rt::test]
async fn test_initialize_user_not_logged_in() {
    let runner = QueryRunner::new();

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "username" => InputValue::scalar("user1"),
                }
            )
            .await,
        (
            serde_json::Value::Null,
            vec![json!({
                "message": "Not logged in",
                "locations": [{"line": 5, "column": 9}],
                "path": ["initializeUser"],
            })]
        )
    );
}

/// Setting a username that's already taken should return an error.
#[actix_rt::test]
async fn test_initialize_user_duplicate_username() {
    let mut runner = QueryRunner::new();
    runner.disable_transaction();

    let user_provider = runner.run_with_conn(|conn| {
        UserFactory::default().username("user1").insert(&conn);
        UserProviderFactory::default().insert(&conn)
    });
    runner.set_user_provider(user_provider);

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "username" => InputValue::scalar("user1"), // Already taken
                }
            )
            .await,
        (
            serde_json::Value::Null,
            vec![json!({
                "message": "This resource already exists",
                "locations": [{"line": 5, "column": 9}],
                "path": ["initializeUser"],
            })]
        )
    );
}

/// Setting a username that doesn't pass validation should return an error
#[actix_rt::test]
async fn test_initialize_user_invalid_username() {
    let mut runner = QueryRunner::new();
    runner.disable_transaction();

    let user_provider = runner
        .run_with_conn(|conn| UserProviderFactory::default().insert(&conn));
    runner.set_user_provider(user_provider);

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "username" => InputValue::scalar(""), // Invalid username
                }
            )
            .await,
        (
            serde_json::Value::Null,
            vec![json!({
                "message": "Input validation error(s)",
                "locations": [{"line": 5, "column": 9}],
                "path": ["initializeUser"],
                "extensions": {
                    "username": [
                        {"min": "1", "value": "\"\"", "max": "20"},
                    ]
                },
            })]
        )
    );
    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    // Length limit is 20 chars
                    "username" => InputValue::scalar("012345678901234567890"),
                }
            )
            .await,
        (
            serde_json::Value::Null,
            vec![json!({
                "message": "Input validation error(s)",
                "locations": [{"line": 5, "column": 9}],
                "path": ["initializeUser"],
                "extensions": {
                    "username": [
                        {"min": "1", "value": "\"012345678901234567890\"", "max": "20"},
                    ]
                },
            })]
        )
    );
}

/// Trying to initialize a user that's already been initialized should return
/// an error.
#[actix_rt::test]
async fn test_initialize_user_already_initialized() {
    let mut runner = QueryRunner::new();
    runner.disable_transaction();

    let user_provider = runner.run_with_conn(|conn| {
        let user = UserFactory::default().username("user1").insert(&conn);
        UserProviderFactory::default()
            .user(Some(&user))
            .insert(&conn)
    });
    runner.set_user_provider(user_provider);

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "username" => InputValue::scalar("user2"),
                }
            )
            .await,
        (
            serde_json::Value::Null,
            vec![json!({
                // This is a shitty error message, but fuck it. This shouldn't
                // be possible to hit from the UI anyway
                "message": "Not found",
                "locations": [{"line": 5, "column": 9}],
                "path": ["initializeUser"],
            })]
        )
    );

    // Make sure there's still only one user in the DB, to ensure that the
    // user creation got rolled back
    runner.run_with_conn(|conn| {
        assert_eq!(
            users::table
                .select(dsl::count_star())
                .get_result::<i64>(conn)
                .unwrap(),
            1
        );
    });
}
