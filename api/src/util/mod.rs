//! This module holds both general and niche utility functions and types.
//! More general/miscellaneous things are in this file, while more specific
//! sub-categories live in submodules. Everything is exported directly from this
//! module though.

mod auth;
mod db;
#[cfg(test)]
mod tests;

pub use auth::*;
pub use db::*;
use std::ops::Deref;
#[cfg(test)]
pub use tests::*;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

/// A small wrapper struct to indicate that the wrapped value has been
/// validated. Built on top of [validator]. This struct can only be constructed
/// via [Valid::validate].
#[derive(Copy, Clone, Debug)]
pub struct Valid<T: Validate> {
    inner: T,
}

impl<T: Validate> Valid<T> {
    /// Validate the given value. If validation succeeds, wrap it in a
    /// [Valid] to indicate it's valid.
    pub fn validate(value: T) -> Result<Self, ValidationErrors> {
        // We can't do a blanket TryFrom<T: Validate> implementation because of
        // this bug https://github.com/rust-lang/rust/issues/50133
        // Will have to wait for better specialization
        value.validate()?;
        Ok(Self { inner: value })
    }
}

impl<T: Validate> Deref for Valid<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

/// Converts a UUID to a Juniper (GraphQL) ID.
pub fn uuid_to_gql_id(uuid: Uuid) -> juniper::ID {
    juniper::ID::new(uuid.to_string())
}

/// Converts a Juniper (GraphQL) ID to a UUID. If the given string is not a
/// valid UUID, then just return the nil UUID (all zeroes). This is useful in
/// the API because we want to handle malformed UUIDs the same way we handle
/// UUIDs that aren't in the database.
pub fn gql_id_to_uuid(id: &juniper::ID) -> Uuid {
    parse_uuid(&id.to_string())
}

/// Parses the given string into a UUID. If the string cannot be parsed
/// properly, this just returns the default UUID (all zeroes).
pub fn parse_uuid(id: &str) -> Uuid {
    Uuid::parse_str(id).unwrap_or_default()
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
