//! A helper module to hold utilities that are used across tests. This file
//! DOES NOT container any of its own tests.

use diesel::{associations::HasTable, PgConnection, RunQueryDsl};
use gdlk_api::{
    models,
    schema::user_providers,
    server::{create_gql_schema, Context, GqlSchema, UserContext},
    util::{self, PooledConnection},
};
use juniper::{ExecutionError, InputValue, Variables};
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};

/// Macro to run a DELETE statement for each given table. Used to clean up all
/// tables after each test.
macro_rules! delete_tables {
    ($conn:expr, $($model:ty),+ $(,)?) => {
        $(
            diesel::delete(<$model>::table())
            .execute($conn as &PgConnection)
            .unwrap();
        )+
    };
}

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
        let context = Context {
            pool: Arc::new(util::init_db_conn_pool().unwrap()),
            user_context: None,
        };
        Self {
            schema: create_gql_schema(),
            context,
        }
    }

    /// Get a DB connection from the pool.
    pub fn db_conn(&self) -> PooledConnection {
        self.context.get_db_conn().unwrap()
    }

    /// Modify the query context to set the current user. Creates a placeholder
    /// UserProvider to be used for authentication. This should be used for
    /// most tests that require a user.
    #[allow(dead_code)] // Not all crates use this
    pub fn set_user(&mut self, user: models::User) {
        // Create a bogus user_provider for this user. We're not trying to test
        // the OpenID logic here, so this is fine.
        let user_provider_id = models::NewUserProvider {
            sub: &user.id.to_string(), // guarantees uniqueness
            provider_name: "fake_provider",
            user_id: Some(user.id),
        }
        .insert()
        .returning(user_providers::columns::id)
        .get_result(&self.db_conn())
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

// Automatically wipe all tables when the test is done
impl Drop for QueryRunner {
    fn drop(&mut self) {
        let conn = self.context.get_db_conn().unwrap();
        delete_tables!(
            &conn,
            models::UserProvider,
            models::UserProgram,
            models::ProgramSpec,
            models::HardwareSpec,
            models::User,
            // Any new table needs to be added here!
        );
    }
}
