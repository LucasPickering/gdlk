//! General utility functions and types.

#[cfg(test)]
pub use tests::*;

#[cfg(test)]
mod tests {
    use diesel::{Connection, PgConnection};

    /// Helper to create a database connection for testing. This establishes
    /// the connection, then starts a test transaction on it so that no changes
    /// will actually be written to the DB.
    pub fn test_connection() -> PgConnection {
        let database_url = std::env::var("DATABASE_URL").unwrap();
        let conn = PgConnection::establish(&database_url).unwrap();
        conn.begin_test_transaction().unwrap();
        conn
    }
}
