use crate::{models::sql_types::PermissionType, schema::permissions};
use diesel::{Identifiable, Queryable};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Identifiable, Queryable)]
#[table_name = "permissions"]
pub struct Permission {
    pub id: Uuid,
    pub name: PermissionType,
}
