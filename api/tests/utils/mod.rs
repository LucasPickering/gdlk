//! A helper module to hold utilities that are used across tests. This file
//! DOES NOT container any of its own tests.

use diesel::{PgConnection, RunQueryDsl};
use gdlk_api::{
    models::{self, Factory},
    schema::user_providers,
    server::{create_gql_schema, Context, GqlSchema, UserContext},
    util,
};
use juniper::{ExecutionError, InputValue, Variables};
use serde::Serialize;
use std::collections::HashMap;

/// Convert a serializable value into a JSON value.
pub fn to_json<T: Serialize>(input: T) -> serde_json::Value {
    let serialized: String = serde_json::to_string(&input).unwrap();
    serde_json::from_str(&serialized).unwrap()
}

/// Helper type for setting up and executing test GraphQL queries
pub struct QueryRunner {
    schema: GqlSchema,
    context: Context,
}

impl QueryRunner {
    pub fn new() -> Self {
        let pool = util::init_test_db_conn_pool().unwrap();
        let context = Context {
            db_conn: pool.get().unwrap(),
            user_context: None,
        };
        Self {
            schema: create_gql_schema(),
            context,
        }
    }

    /// Get a DB connection from the pool.
    pub fn db_conn(&self) -> &PgConnection {
        self.context.db_conn()
    }

    /// Modify the query context to set the current user. Creates a placeholder
    /// UserProvider to be used for authentication. For most tests you can just
    /// use [Self::log_in], this is only necessary when you need more control
    /// over the logged-in user.
    pub fn set_user(&mut self, user: &models::User) {
        // Create a bogus user_provider for this user. We're not trying to test
        // the OpenID logic here, so this is fine.
        let user_provider_id = models::NewUserProvider {
            sub: &user.id.to_string(), // guarantees uniqueness
            provider_name: "fake_provider",
            user_id: Some(user.id),
        }
        .insert()
        .returning(user_providers::columns::id)
        .get_result(self.db_conn())
        .unwrap(); // Failure here indicates some unexpected DB/network error

        self.context.user_context = Some(UserContext {
            user_provider_id,
            user_id: Some(user.id),
        });
    }

    /// Modify the query context to set the current user_provider and user. This
    /// is useful when testing the auth functionality, but if you just need
    /// an authenticated user, then you probably want to use [Self::set_user].
    #[allow(dead_code)] // Not all crates use this
    pub fn set_user_provider(&mut self, user_provider: models::UserProvider) {
        self.context.user_context = Some(UserContext {
            user_provider_id: user_provider.id,
            user_id: user_provider.user_id,
        })
    }

    /// Create a new user and set them as the logged-in user. This is the
    /// easiest way to log in for a test, and should be used for most tests.
    #[allow(dead_code)] // Not all crates use this
    pub fn log_in(&mut self) -> models::User {
        let user =
            models::NewUser { username: "user1" }.create(&self.db_conn());
        self.set_user(&user);
        user
    }

    pub fn query<'a>(
        &'a self,
        query: &'a str,
        vars: HashMap<&str, InputValue>,
    ) -> (serde_json::Value, Vec<serde_json::Value>) {
        // Map &strs to Strings
        let converted_vars = vars
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect::<Variables>();

        let (data, errors): (juniper::Value<_>, Vec<ExecutionError<_>>) =
            juniper::execute(
                query,
                None,
                &self.schema,
                &converted_vars,
                &self.context,
            )
            .unwrap();

        // Map the output data to JSON, for easier comparison
        (to_json(data), errors.into_iter().map(to_json).collect())
    }
}
