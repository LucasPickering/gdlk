use crate::{
    error::{ResponseError, ServerError},
    schema::permissions,
};
use diesel::{Identifiable, Queryable};
use std::str::FromStr;
use uuid::Uuid;

/// A mapping of all variants to the corresponding name used in the DB
const NAME_MAPPING: &[(PermissionType, &str)] = &[
    (PermissionType::CreateSpecs, "Create Specs"),
    (PermissionType::ModifyAllSpecs, "Modify All Specs"),
    (PermissionType::DeleteAllSpecs, "Delete All Specs"),
    (
        PermissionType::ViewAllUserPrograms,
        "View All User Programs",
    ),
];

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum PermissionType {
    /// User can create new hardware and program specs
    CreateSpecs,

    /// User can modify all existing hardware and program specs, regardless of
    /// whether or not they are the owner.
    ModifyAllSpecs,

    /// User can delete all existing hardware and program specs, regardless of
    /// whether or not they are the owner.
    DeleteAllSpecs,

    /// User can view all existing user programs, not just their own.
    ViewAllUserPrograms,
}

impl PermissionType {
    pub fn to_str(self) -> &'static str {
        for (permission_type, name) in NAME_MAPPING {
            if self == *permission_type {
                return name;
            }
        }
        panic!("Missing name for permission type: {:?}", self);
    }
}

impl FromStr for PermissionType {
    type Err = ResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for (permission_type, name) in NAME_MAPPING {
            if s == *name {
                return Ok(*permission_type);
            }
        }

        // Unknown value
        Err(ServerError::InvalidDbValue {
            column: Box::new(permissions::columns::name),
            value: s.into(),
        }
        .into())
    }
}

#[derive(Clone, Debug, PartialEq, Identifiable, Queryable)]
#[table_name = "permissions"]
pub struct Permission {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util;
    use diesel::{QueryDsl, RunQueryDsl};

    /// Test that the list of permission types we define here in code matches
    /// the list of types that exist in the DB. All permissions are created by
    /// migrations, so we don't need to seed any data here.
    #[test]
    fn test_all_permission_types() {
        let pool = util::init_test_db_conn_pool().unwrap();
        let conn = &pool.get().unwrap();
        let all_permissions: Vec<Permission> = permissions::table
            .select(permissions::all_columns)
            // sort so we can do stable comparisons later
            .order_by(permissions::columns::name)
            .get_results(conn)
            .unwrap();

        // Test that all names can be deserialized correctly
        for permission in all_permissions.iter() {
            if let Err(err) = permission.name.parse::<PermissionType>() {
                panic!(
                    "Unable to parse permission name: {}\n{}",
                    permission.name, err
                );
            }
        }

        let queried_names: Vec<&str> = all_permissions
            .iter()
            .map(|permission| permission.name.as_str())
            .collect();

        // Test that all names can be serialized correctly
        let mut serialized_names: Vec<&str> = NAME_MAPPING
            .iter()
            .map(|(permission_type, _)| permission_type.to_str())
            .collect();
        serialized_names.sort();

        // Make sure the list of serialized names matches all the names we
        // pulled from the DB. This makes sure we don't have any extraneous
        // variants defined in code.
        assert_eq!(
            queried_names, serialized_names,
            "Serialized permission types did not match names from the DB"
        );
    }
}
