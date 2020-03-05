use crate::{
    models::{HardwareSpec, User},
    schema::{program_specs, user_programs},
};
use diesel::{
    prelude::*, query_builder::InsertStatement, Identifiable, Queryable,
};
use gdlk::{ast::LangValue, validator::ValidationErrors, Valid};
use std::convert::TryFrom;
use uuid::Uuid;

/// A derivative of [ProgramSpec](gdlk::ProgramSpec), built from a DB query.
#[derive(Clone, Debug, PartialEq, Identifiable, Associations, Queryable)]
#[belongs_to(HardwareSpec, foreign_key = "hardware_spec_id")]
#[table_name = "program_specs"]
pub struct ProgramSpec {
    pub id: Uuid,
    /// Space-less identifier, unique to all program specs for a particular
    /// hardware spec (i.e. unique with `hardware_spec_id`)
    pub slug: String,
    /// ID of the hardware that this program runs on
    pub hardware_spec_id: Uuid,
    /// The input values, where the element at position 0 is the first one that
    /// will be popped off.
    pub input: Vec<LangValue>,
    /// The correct value to be left in the output when the program exits. The
    /// first element will be the first one pushed, and so on.
    pub expected_output: Vec<LangValue>,
}

impl TryFrom<ProgramSpec> for Valid<gdlk::ProgramSpec> {
    type Error = ValidationErrors;

    fn try_from(other: ProgramSpec) -> Result<Self, Self::Error> {
        Valid::validate(gdlk::ProgramSpec {
            input: other.input,
            expected_output: other.expected_output,
        })
    }
}

/// A derivative of [ProgramSpec](gdlk::ProgramSpec), meant for DB inserts.
/// This can be constructed manually and inserted into the DB. These fields
/// all correspond to [ProgramSpec](ProgramSpec), so look there for
/// field-level documentation.
#[derive(Debug, PartialEq, Insertable)]
#[table_name = "program_specs"]
pub struct NewProgramSpec<'a> {
    pub slug: &'a str,
    pub hardware_spec_id: Uuid,
    pub input: Vec<LangValue>,
    pub expected_output: Vec<LangValue>,
}

impl NewProgramSpec<'_> {
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

#[derive(Debug, PartialEq, Queryable, Associations)]
#[belongs_to(User, foreign_key = "user_id")]
#[belongs_to(ProgramSpec, foreign_key = "program_spec_id")]
#[table_name = "user_programs"]
pub struct UserProgram {
    pub id: Uuid,
    pub user_id: Uuid,
    pub program_spec_id: Uuid,
    pub file_name: String,
    pub source_code: String,
}

#[derive(Debug, PartialEq, Insertable)]
#[table_name = "user_programs"]
pub struct NewUserProgram<'a> {
    pub user_id: Uuid,
    pub program_spec_id: Uuid,
    pub file_name: &'a str,
    pub source_code: &'a str,
}

impl NewUserProgram<'_> {
    /// Insert this object into the `user_programs` DB table.
    pub fn insert(
        self,
    ) -> InsertStatement<
        user_programs::table,
        <Self as Insertable<user_programs::table>>::Values,
    > {
        self.insert_into(user_programs::table)
    }
}
