use crate::{
    models::sql_types::{RoleType, Role_type},
    schema::roles,
};
use diesel::{
    dsl, expression::bound::Bound, ExpressionMethods, Identifiable, QueryDsl,
    Queryable,
};
use uuid::Uuid;

/// Expression to filter roles by name
pub type WithName = dsl::Eq<roles::columns::name, Bound<Role_type, RoleType>>;

#[derive(Clone, Debug, PartialEq, Identifiable, Queryable)]
#[table_name = "roles"]
pub struct Role {
    pub id: Uuid,
    pub name: RoleType,
    pub is_admin: bool,
}

impl Role {
    pub fn filter_by_name(
        name: RoleType,
    ) -> dsl::Filter<roles::table, WithName> {
        roles::table.filter(roles::columns::name.eq(name))
    }
}
