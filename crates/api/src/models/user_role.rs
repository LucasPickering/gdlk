use crate::schema::user_roles;
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

#[derive(Debug, PartialEq, Insertable)]
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
