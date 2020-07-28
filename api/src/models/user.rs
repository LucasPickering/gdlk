use crate::{
    models::{Factory, RoleType},
    schema::{roles, user_roles, users},
};
use diesel::{
    dsl, expression::bound::Bound, prelude::*, query_builder::InsertStatement,
    sql_types, Identifiable, Queryable,
};
use uuid::Uuid;
use validator::Validate;

/// Expression to filter users by username
pub type WithUsername<'a> =
    dsl::Eq<users::columns::username, Bound<sql_types::Text, &'a str>>;

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

    /// Build and execute query to grant each of the given roles to the user.
    pub fn add_roles_x(
        &self,
        conn: &PgConnection,
        roles: &[RoleType],
    ) -> Result<usize, diesel::result::Error> {
        // Map the role types to strings. No need to collect this iter, just
        // pass it along directly
        let role_names = roles.iter().map(|role_type| role_type.to_str());
        diesel::insert_into(user_roles::table)
            .values(
                roles::table
                    .select((
                        self.id.into_sql::<sql_types::Uuid>(),
                        roles::columns::id,
                    ))
                    .filter(roles::columns::name.eq_any(role_names)),
            )
            .into_columns((
                user_roles::columns::user_id,
                user_roles::columns::role_id,
            ))
            .execute(conn)
    }
}

#[derive(Debug, Default, PartialEq, Insertable, Validate)]
#[table_name = "users"]
pub struct NewUser<'a> {
    #[validate(length(min = 1, max = 20))]
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
