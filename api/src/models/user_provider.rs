use crate::{
    models::{Factory, User},
    schema::user_providers,
};
use diesel::{prelude::*, query_builder::InsertStatement, Queryable};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Identifiable, Associations, Queryable)]
#[belongs_to(User, foreign_key = "user_id")]
#[table_name = "user_providers"]
pub struct UserProvider {
    pub id: Uuid,
    pub sub: String,
    pub provider_name: String,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Default, PartialEq, Insertable)]
#[table_name = "user_providers"]
pub struct NewUserProvider<'a> {
    pub sub: &'a str,
    pub provider_name: &'a str,
    pub user_id: Option<Uuid>,
}

impl NewUserProvider<'_> {
    /// Insert this object into the `user_providers` DB table.
    pub fn insert(
        self,
    ) -> InsertStatement<
        user_providers::table,
        <Self as Insertable<user_providers::table>>::Values,
    > {
        self.insert_into(user_providers::table)
    }
}

// This trait is only needed for tests
impl Factory for NewUserProvider<'_> {
    type ReturnType = UserProvider;

    fn create(self, conn: &PgConnection) -> UserProvider {
        self.insert()
            .returning(user_providers::all_columns)
            .get_result(conn)
            .unwrap()
    }
}
