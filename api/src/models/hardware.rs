use crate::schema::hardware_specs;
use diesel::{
    dsl, expression::bound::Bound, prelude::*, query_builder::InsertStatement,
    sql_types::Text, Identifiable, Queryable,
};
use gdlk::{
    validator::{Validate, ValidationErrors},
    Valid,
};
use std::convert::TryFrom;

/// Eq clause to filter hardware_specs by slug
pub type WithSlug<'a> =
    dsl::Eq<hardware_specs::columns::slug, Bound<Text, &'a str>>;

/// A derivative of [HardwareSpec](gdlk::HardwareSpec), containing all fields.
/// that are present on the DB table. This should only ever be constructed from
/// a DB query.
#[derive(Debug, PartialEq, Identifiable, Queryable)]
#[table_name = "hardware_specs"]
pub struct HardwareSpec {
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

impl HardwareSpec {
    /// Filters hardware specs by their slug.
    pub fn filter_by_slug<'a>(
        hw_spec_slug: &'a str,
    ) -> dsl::Filter<hardware_specs::table, WithSlug<'a>> {
        hardware_specs::dsl::hardware_specs
            .filter(hardware_specs::dsl::slug.eq(hw_spec_slug))
    }
}

impl TryFrom<HardwareSpec> for gdlk::HardwareSpec {
    type Error = ValidationErrors;

    fn try_from(other: HardwareSpec) -> Result<Self, Self::Error> {
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
