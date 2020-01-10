use crate::schema::hardware_specs;
use diesel::{
    prelude::*, query_builder::InsertStatement, Identifiable, Queryable,
};
use gdlk::HardwareSpec;
use std::convert::TryFrom;
use validator::{Validate, ValidationErrors};

/// A derivative of [HardwareSpec](gdlk::HardwareSpec), containing all fields.
/// that are present on the DB table. This should only ever be constructed from
/// a DB query.
#[derive(Debug, PartialEq, Identifiable, Queryable)]
#[table_name = "hardware_specs"]
pub struct FullHardwareSpec {
    /// DB row ID
    pub id: i32,
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

impl TryFrom<FullHardwareSpec> for HardwareSpec {
    type Error = ValidationErrors;

    fn try_from(other: FullHardwareSpec) -> Result<Self, Self::Error> {
        let val = Self {
            // These conversions are safe because of the .validate() call below.
            // The validation _should_ never fail because of the DB constraints,
            // but we validate here just to be safe
            num_registers: other.num_registers as usize,
            num_stacks: other.num_stacks as usize,
            max_stack_length: other.max_stack_length as usize,
        };
        val.validate()?;
        Ok(val)
    }
}

/// A derivative of [HardwareSpec](gdlk::HardwareSpec), meant for DB inserts.
/// This can be constructed manually and inserted into the DB. These fields
/// all correspond to [FullHardwareSpec](FullHardwareSpec), so look there for
/// field-level documentation.
#[derive(Debug, PartialEq, Insertable)]
#[table_name = "hardware_specs"]
pub struct NewHardwareSpec {
    pub slug: String,
    pub num_registers: i32,
    pub num_stacks: i32,
    pub max_stack_length: i32,
}

impl NewHardwareSpec {
    pub fn insert(
        self,
    ) -> InsertStatement<
        hardware_specs::table,
        <Self as Insertable<hardware_specs::table>>::Values,
    > {
        self.insert_into(hardware_specs::table)
    }
}
