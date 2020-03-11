use crate::schema::users;
use diesel::{
    dsl, expression::bound::Bound, prelude::*, query_builder::InsertStatement,
    sql_types::Text, Identifiable, Queryable,
};
use uuid::Uuid;

/// Expression to filter users by username
type WithUsername<'a> = dsl::Eq<users::columns::username, Bound<Text, &'a str>>;

#[derive(Clone, Debug, PartialEq, Identifiable, Queryable)]
#[table_name = "users"]
pub struct User {
    pub id: Uuid,
    pub username: String,
}

impl User {
    /// Filters users by their username. The resulting queryset should contain
    /// no more than one user.
    pub fn filter_by_username<'a>(
        username: &'a str,
    ) -> dsl::Filter<users::table, WithUsername<'a>> {
        users::table.filter(users::dsl::username.eq(username))
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
