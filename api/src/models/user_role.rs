use crate::{models::Factory, schema::user_roles};
use diesel::{
    prelude::*, query_builder::InsertStatement, Identifiable, Queryable,
};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Identifiable, Queryable)]
#[table_name = "user_roles"]
pub struct UserRole {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[derive(Debug, Default, PartialEq, Insertable)]
#[table_name = "user_roles"]
pub struct NewUserRole {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

impl NewUserRole {
    /// Insert this object into the `user_roles` DB table.
    pub fn insert(
        self,
    ) -> InsertStatement<
        user_roles::table,
        <Self as Insertable<user_roles::table>>::Values,
    > {
        self.insert_into(user_roles::table)
    }
}

// This trait is only needed for tests
impl Factory for NewUserRole {
    type ReturnType = UserRole;

    fn create(self, conn: &PgConnection) -> UserRole {
        self.insert()
            .returning(user_roles::all_columns)
            .get_result(conn)
            .unwrap()
    }
}
