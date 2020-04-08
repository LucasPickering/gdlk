//! General utility functions and types.

#[cfg(test)]
pub use self::tests::*;
use diesel::{r2d2::ConnectionManager, PgConnection};
use failure::Fallible;
use uuid::{parser::ParseError, Uuid};

/// Type aliases for DB connections
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type PooledConnection =
    r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub fn init_db_conn_pool() -> Fallible<Pool> {
    let database_url = std::env::var("DATABASE_URL")?;
    let manager = ConnectionManager::new(database_url);
    let pool = r2d2::Pool::builder().build(manager)?;
    Ok(pool)
}

/// Converts a UUID to a Juniper (GraphQL) ID.
pub fn uuid_to_gql_id(uuid: Uuid) -> juniper::ID {
    juniper::ID::new(uuid.to_string())
}

/// Converts a Juniper (GraphQL) ID to a UUID.
pub fn gql_id_to_uuid(id: &juniper::ID) -> Result<Uuid, ParseError> {
    Uuid::parse_str(&id.to_string())
}

/// Converts a map to a GraphQL object. Takes in an iterator of (K, V) so that
/// any map type can be used (HashMap, BTreeMap, etc.).
pub fn map_to_gql_object<K: ToString, V>(
    map: impl ExactSizeIterator<Item = (K, V)>,
    mapper: impl Fn(V) -> juniper::Value,
) -> juniper::Value {
    let len = map.len();
    let obj = map.fold(
        juniper::Object::with_capacity(len),
        |mut acc, (field, value)| {
            acc.add_field(field.to_string(), mapper(value));
            acc
        },
    );
    juniper::Value::Object(obj)
}

#[cfg(test)]
mod tests {
    /// Assert that the first value is an Err, and that its string form matches
    /// the second argument.
    #[macro_export]
    macro_rules! assert_err {
        ($res:expr, $msg:tt $(,)?) => {
            match $res {
                Ok(_) => panic!("Expected Err, got Ok"),
                Err(err) => assert_eq!(err.to_string(), $msg),
            }
        };
    }
}
