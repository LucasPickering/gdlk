use crate::{models::Factory, schema::users};
use diesel::{
    dsl, expression::bound::Bound, prelude::*, query_builder::InsertStatement,
    sql_types::Text, Identifiable, Queryable,
};
use uuid::Uuid;
use validator::Validate;

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

    /// Start a query that filters by username.
    pub fn filter_by_username(
        username: &str,
    ) -> dsl::Filter<users::table, WithUsername<'_>> {
        users::table.filter(Self::with_username(username))
    }
}

#[derive(Debug, Default, PartialEq, Insertable, Validate)]
#[table_name = "users"]
pub struct NewUser<'a> {
    #[validate(length(min = 1))]
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

// This trait is only needed for tests
impl Factory for NewUser<'_> {
    type ReturnType = User;

    fn create(self, conn: &PgConnection) -> User {
        self.insert()
            .returning(users::all_columns)
            .get_result(conn)
            .unwrap()
    }
}
