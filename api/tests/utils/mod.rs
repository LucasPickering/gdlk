//! A helper module to hold utilities that are used across tests. This file
//! DOES NOT container any of its own tests.

use diesel::{associations::HasTable, PgConnection, RunQueryDsl};
use failure::Fallible;
use gdlk_api::{
    models,
    server::{create_gql_schema, Context, GqlSchema},
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
    pub fn new() -> Fallible<Self> {
        let pool = util::init_db_conn_pool()?;

        Ok(Self {
            schema: create_gql_schema(),
            context: Context {
                pool: Arc::new(pool),
            },
        })
    }

    pub fn db_conn(&self) -> PooledConnection {
        self.context.get_db_conn().unwrap()
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
            models::UserProgram,
            models::ProgramSpec,
            models::HardwareSpec,
            models::User,
            // Any new table needs to be added here!
        );
    }
}
