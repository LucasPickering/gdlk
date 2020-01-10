use crate::{models::hardware::FullHardwareSpec, schema::program_specs};
use diesel::{
    prelude::*, query_builder::InsertStatement, Identifiable, Queryable,
};
use gdlk::{ast::LangValue, ProgramSpec};
use std::convert::TryFrom;
use validator::{Validate, ValidationErrors};

/// A derivative of [ProgramSpec](gdlk::ProgramSpec), built from a DB query.
#[derive(Debug, PartialEq, Identifiable, Associations, Queryable)]
#[belongs_to(FullHardwareSpec, foreign_key = "hardware_spec_id")]
#[table_name = "program_specs"]
pub struct FullProgramSpec {
    pub id: i32,
    /// Space-less identifier, unique to all program specs for a particular
    /// hardware spec (i.e. unique with `hardware_spec_id`)
    pub slug: String,
    /// ID of the hardware that this program runs on
    pub hardware_spec_id: i32,
    /// The input values, where the element at position 0 is the first one that
    /// will be popped off.
    pub input: Vec<LangValue>,
    /// The correct value to be left in the output when the program exits. The
    /// first element will be the first one pushed, and so on.
    pub expected_output: Vec<LangValue>,
}

impl TryFrom<FullProgramSpec> for ProgramSpec {
    type Error = ValidationErrors;

    fn try_from(other: FullProgramSpec) -> Result<Self, Self::Error> {
        let val = Self {
            input: other.input,
            expected_output: other.expected_output,
        };
        val.validate()?;
        Ok(val)
    }
}

/// A derivative of [ProgramSpec](gdlk::ProgramSpec), meant for DB inserts.
/// This can be constructed manually and inserted into the DB. These fields
/// all correspond to [FullProgramSpec](FullProgramSpec), so look there for
/// field-level documentation.
#[derive(Debug, PartialEq, Insertable)]
#[table_name = "program_specs"]
pub struct NewProgramSpec {
    pub slug: String,
    pub hardware_spec_id: i32,
    pub input: Vec<LangValue>,
    pub expected_output: Vec<LangValue>,
}

impl NewProgramSpec {
    /// Insert this object into the `program_specs` DB table.
    pub fn insert(
        self,
    ) -> InsertStatement<
        program_specs::table,
        <Self as Insertable<program_specs::table>>::Values,
    > {
        self.insert_into(program_specs::table)
    }
}
