use crate::schema::users;
use diesel::{
    dsl, expression::bound::Bound, prelude::*, query_builder::InsertStatement,
    sql_types::Text, Identifiable, Queryable,
};
use uuid::Uuid;

/// Expression to filter users by username
pub type WithUsername<'a> =
    dsl::Eq<users::columns::username, Bound<Text, &'a str>>;

#[derive(Clone, Debug, PartialEq, Identifiable, Queryable)]
#[table_name = "users"]
pub struct User {
    pub id: Uuid,
    pub username: String,
}

impl User {
    /// Eq clause to compare the username column to a value.
    pub fn with_username(username: &str) -> WithUsername<'_> {
        users::dsl::username.eq(username)
    }
}

#[derive(Debug, PartialEq, Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
}

impl NewUser<'_> {
    /// Insert this object into the `users` DB table.
    pub fn insert(
        self,
    ) -> InsertStatement<users::table, <Self as Insertable<users::table>>::Values>
    {
        self.insert_into(users::table)
    }
}
