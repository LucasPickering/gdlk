use crate::{models::Factory, schema::hardware_specs};
use diesel::{
    prelude::*, query_builder::InsertStatement, Identifiable, Queryable,
};
use gdlk::{validator::ValidationErrors, Valid};
use std::convert::TryFrom;
use uuid::Uuid;

/// A derivative of [HardwareSpec](gdlk::HardwareSpec), containing all fields.
/// that are present on the DB table. This should only ever be constructed from
/// a DB query.
#[derive(Clone, Debug, PartialEq, Identifiable, Queryable)]
#[table_name = "hardware_specs"]
pub struct HardwareSpec {
    /// DB row ID
    pub id: Uuid,
    /// Unique user-friendly identifier that can be used in URLs
    pub slug: String,

    // These three need to be i32s because postgres has no unsigned type.
    // The insertion code and DB should both enforce that they are >= 0.
    /// Number of registers available
    pub num_registers: i32,
    /// Maximum number of stacks permitted
    pub num_stacks: i32,
    /// Maximum size of each stack
    pub max_stack_length: i32,
}

impl TryFrom<HardwareSpec> for Valid<gdlk::HardwareSpec> {
    type Error = ValidationErrors;

    fn try_from(other: HardwareSpec) -> Result<Self, Self::Error> {
        Valid::validate(gdlk::HardwareSpec {
            // These conversions are safe because of the validate() call.
            // The validation _should_ never fail because of the DB constraints,
            // but we validate here just to be safe.
            num_registers: other.num_registers as usize,
            num_stacks: other.num_stacks as usize,
            max_stack_length: other.max_stack_length as usize,
        })
    }
}

/// A derivative of [HardwareSpec](gdlk::HardwareSpec), meant for DB inserts.
/// This can be constructed manually and inserted into the DB. These fields
/// all correspond to [HardwareSpec](HardwareSpec), so look there for
/// field-level documentation.
#[derive(Debug, PartialEq, Insertable)]
#[table_name = "hardware_specs"]
pub struct NewHardwareSpec<'a> {
    pub slug: &'a str,
    pub num_registers: i32,
    pub num_stacks: i32,
    pub max_stack_length: i32,
}

impl NewHardwareSpec<'_> {
    pub fn insert(
        self,
    ) -> InsertStatement<
        hardware_specs::table,
        <Self as Insertable<hardware_specs::table>>::Values,
    > {
        self.insert_into(hardware_specs::table)
    }
}

// This trait is only needed for tests
impl Factory for NewHardwareSpec<'_> {
    type ReturnType = HardwareSpec;

    fn create(self, conn: &PgConnection) -> HardwareSpec {
        self.insert()
            .returning(hardware_specs::all_columns)
            .get_result(conn)
            .unwrap()
    }
}
