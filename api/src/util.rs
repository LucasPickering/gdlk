//! General utility functions and types.

#[cfg(test)]
pub use tests::*;

use diesel::{r2d2::ConnectionManager, PgConnection};

/// Type aliases for DB connections
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type PooledConnection =
    r2d2::PooledConnection<ConnectionManager<PgConnection>>;

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::{Connection, PgConnection};

    /// Helper to create a database connection for testing. This establishes
    /// the connection, then starts a test transaction on it so that no changes
    /// will actually be written to the DB.
    pub fn test_db_conn() -> PooledConnection {
        let database_url = std::env::var("DATABASE_URL").unwrap();

        // We want to build a connection pool so that we can pass into APIs
        // that expect owned, pooled connections. The pool will also
        // automatically close our connections for us.
        let manager = diesel::r2d2::ConnectionManager::new(&database_url);
        let pool = r2d2::Pool::builder().max_size(5).build(manager).unwrap();
        let conn = pool.get().unwrap();

        (&conn as &PgConnection).begin_test_transaction().unwrap();
        conn
    }
}
