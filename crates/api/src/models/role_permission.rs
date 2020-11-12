use crate::schema::role_permissions;
use diesel::{Identifiable, Queryable};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Identifiable, Queryable)]
#[table_name = "role_permissions"]
pub struct RolePermission {
    pub id: Uuid,
    pub role_id: Uuid,
    pub permissions_id: Uuid,
}
