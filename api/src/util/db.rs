//! Utility functions and types related to the database.

use diesel::{r2d2::ConnectionManager, Connection, PgConnection};
use r2d2::CustomizeConnection;

/// Type aliases for DB connections
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type PooledConnection =
    r2d2::PooledConnection<ConnectionManager<PgConnection>>;

/// A DB connection customizer that wraps each connection in a transaction
/// before returning it. This should be used in all unit tests to prevent
/// make changes to the DB.
#[derive(Copy, Clone, Debug)]
struct TestConnectionCustomizer;

impl CustomizeConnection<PgConnection, diesel::r2d2::Error>
    for TestConnectionCustomizer
{
    fn on_acquire(
        &self,
        conn: &mut PgConnection,
    ) -> Result<(), diesel::r2d2::Error> {
        conn.begin_test_transaction()
            .map_err(diesel::r2d2::Error::QueryError)?;
        Ok(())
    }

    fn on_release(&self, _conn: PgConnection) {}
}

/// Initialize a new DB connection pool, for use in the webserver.
pub fn init_db_conn_pool(database_url: &str) -> Result<Pool, r2d2::Error> {
    let manager = ConnectionManager::new(database_url);
    r2d2::Pool::builder().build(manager)
}

/// Initialize a new DB connection pool for use in tests. Reads the DB URL from
/// the environment. Also, all connections are wrapped in a test transaction
/// to prevent making modifications to the DB.
pub fn init_test_db_conn_pool() -> Result<Pool, r2d2::Error> {
    let database_url = std::env::var("DATABASE_URL").unwrap();
    let manager = ConnectionManager::new(database_url);
    r2d2::Pool::builder()
        .connection_customizer(Box::new(TestConnectionCustomizer))
        .build(manager)
}
