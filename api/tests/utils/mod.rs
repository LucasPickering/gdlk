//! A helper module to hold utilities that are used across tests. This file
//! DOES NOT container any of its own tests.

pub mod factories;

use crate::utils::factories::*;
use diesel::{
    connection::TransactionManager, Connection, PgConnection, RunQueryDsl,
};
use diesel_factories::Factory;
use gdlk_api::{
    models,
    schema::{user_providers, users},
    server::{create_gql_schema, GqlSchema},
    util::{self, PooledConnection},
    views::{RequestContext, UserContext},
};
use juniper::{ExecutionError, InputValue, Variables};
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};

/// Convert a serializable value into a JSON value.
#[allow(dead_code)] // Not all test crates use this
pub fn to_json<T: Serialize>(input: T) -> serde_json::Value {
    let serialized: String = serde_json::to_string(&input).unwrap();
    serde_json::from_str(&serialized).unwrap()
}

/// Helper type for setting up and executing test GraphQL queries
#[allow(dead_code)] // Not all test crates use this
pub struct QueryRunner {
    schema: GqlSchema,
    context: RequestContext,
}

impl QueryRunner {
    /// Construct a new QueryRunner, which is used to execute GraphQL queries
    /// from a test.
    #[allow(dead_code)] // Not all test crates use this
    pub fn new() -> Self {
        let db_conn_pool = util::init_test_db_conn_pool().unwrap();
        Self {
            schema: create_gql_schema(),
            context: RequestContext::load_context(Arc::new(db_conn_pool), None)
                .unwrap(),
        }
    }

    /// Get a new DB connection. While in testing the pool only holds a single
    /// connection, so the returned connection **needs to be dropped** before
    /// a new one is requested from the pool! To try to enforce that, this
    /// func is interntionally not public.
    fn db_conn(&self) -> PooledConnection {
        self.context.db_conn().unwrap()
    }

    /// Execute a block of code with a DB connection. Tests run with a single
    /// connection in the pool (to enforce that everything happens inside a
    /// transaction), so this restricts access to that connection. This prevents
    /// us from hanging onto a connection reference longer than its needed,
    /// which would block subsequent code and cause a test failure.
    #[allow(dead_code)] // Not all test crates use this
    pub fn run_with_conn<T>(&self, f: impl FnOnce(&PgConnection) -> T) -> T {
        f(&self.db_conn())
    }

    /// Normally all test connections are initialized within a DB transaction.
    /// This prevents any changes made by tests from affecting the DB outside
    /// that test. In some cases though (e.g. if you have transaction logic
    /// in the code being tested), we don't want the test transaction. In those
    /// cases, you can use this method to disable the transaction. When you do,
    /// [QueryRunner] should clean up any data inserted. (WARNING: right now
    /// it doesn't clean all tables - scroll down for more info)
    #[allow(dead_code)] // Not all test crates use this
    pub fn disable_transaction(&self) {
        let conn = self.db_conn();
        conn.transaction_manager()
            .commit_transaction(&conn)
            .unwrap();
    }

    /// Set the user_provider in the user context. This will update the user
    /// context with that provider ID, so the user field will be re-loaded too.
    #[allow(dead_code)] // Not all test crates use this
    pub fn set_user_provider(&mut self, user_provider: models::UserProvider) {
        let conn = self.db_conn();
        self.context.user_context =
            UserContext::load_context(&conn, user_provider.id).unwrap();
    }

    /// Create a new user with the given roles, then update the user context
    /// to be authenticated as that new user. Returns the created user.
    #[allow(dead_code)] // Not all test crates use this
    pub fn log_in(&mut self, roles: &[models::RoleType]) -> models::User {
        let conn = self.db_conn();
        let user = UserFactory::default().username("user1").insert(&conn);

        // Create a bogus user_provider for this user. We're not trying to test
        // the OpenID logic here, so this is fine.
        let user_provider = UserProviderFactory::default()
            .sub(&user.id.to_string()) // guarantees uniqueness
            .user(Some(&user))
            .insert(&conn);

        // Insert one row into user_roles for each requested row
        user.add_roles_x(&conn, roles).unwrap();

        self.context.user_context =
            UserContext::load_context(&conn, user_provider.id).unwrap();
        user
    }

    /// Execute a GraphQL query
    #[allow(dead_code)] // Not all test crates use this
    pub async fn query<'a>(
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
            .await
            .unwrap();

        // Map the output data to JSON, for easier comparison
        (to_json(data), errors.into_iter().map(to_json).collect())
    }
}

impl Drop for QueryRunner {
    fn drop(&mut self) {
        // If the test wasn't inside a transaction, then whatever DB changes it
        // made will still be around - we want to clean those up. Ideally we
        // truncate all tables here, but that sounds like a lot of work that I
        // don't wanna do so just sticking with users for now.
        let conn = self.db_conn();
        if (conn.transaction_manager() as &dyn TransactionManager<PgConnection>)
            .get_transaction_depth()
            == 0
        {
            // TODO clean all tables here
            diesel::delete(user_providers::table)
                .execute(&conn)
                .unwrap();
            diesel::delete(users::table).execute(&conn).unwrap();
        }
    }
}
