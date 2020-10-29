use crate::{
    error::{ApiError, ResponseResult, ServerError},
    schema::roles,
};
use diesel::{Identifiable, Queryable};
use std::str::FromStr;
use uuid::Uuid;

/// A mapping of all variants to the corresponding name used in the DB
const NAME_MAPPING: &[(RoleType, &str)] = &[
    (RoleType::Admin, "Admin"),
    (RoleType::SpecCreator, "Spec Creator"),
];

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum RoleType {
    /// These users can perform any action.
    Admin,

    /// These users can create specs. Once we start attaching the creator to
    /// each spec then we can allow these users to modify their own specs, but
    /// for now they can only create them.
    SpecCreator,
}

impl RoleType {
    pub fn to_str(self) -> &'static str {
        for (role_type, name) in NAME_MAPPING {
            if self == *role_type {
                return name;
            }
        }
        panic!("Missing name for role type: {:?}", self);
    }
}

impl FromStr for RoleType {
    type Err = ApiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for (role_type, name) in NAME_MAPPING {
            if s == *name {
                return Ok(*role_type);
            }
        }

        // Unknown value
        Err(ServerError::InvalidDbValue {
            column: Box::new(roles::columns::name),
            value: s.into(),
        }
        .into())
    }
}

#[derive(Clone, Debug, PartialEq, Identifiable, Queryable)]
#[table_name = "roles"]
pub struct Role {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub is_admin: bool,
}

impl Role {
    pub fn role_type(&self) -> ResponseResult<RoleType> {
        self.name.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util;
    use diesel::{QueryDsl, RunQueryDsl};

    /// Test that the list of role types we define here in code matches
    /// the list of types that exist in the DB. All roles are created by
    /// migrations, so we don't need to seed any data here.
    #[test]
    fn test_all_role_types() {
        let pool = util::init_test_db_conn_pool().unwrap();
        let conn = &pool.get().unwrap();
        let all_roles: Vec<Role> = roles::table
            .select(roles::all_columns)
            // sort so we can do stable comparisons later
            .order_by(roles::columns::name)
            .get_results(conn)
            .unwrap();

        // Test that all names can be deserialized correctly
        for role in all_roles.iter() {
            if let Err(err) = role.name.parse::<RoleType>() {
                panic!("Unable to parse role name: {}\n{}", role.name, err);
            }
        }

        let queried_names: Vec<&str> =
            all_roles.iter().map(|role| role.name.as_str()).collect();

        // Test that all names can be serialized correctly
        let mut serialized_names: Vec<&str> = NAME_MAPPING
            .iter()
            .map(|(role_type, _)| role_type.to_str())
            .collect();
        serialized_names.sort_unstable();

        // Make sure the list of serialized names matches all the names we
        // pulled from the DB. This makes sure we don't have any extraneous
        // variants defined in code.
        assert_eq!(
            queried_names, serialized_names,
            "Serialized role types did not match names from the DB"
        );
    }
}
