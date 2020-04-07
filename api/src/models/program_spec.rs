use crate::{models::HardwareSpec, schema::program_specs};
use diesel::{
    dsl, expression::bound::Bound, prelude::*, query_builder::InsertStatement,
    sql_types, Identifiable, Queryable,
};
use gdlk::{ast::LangValue, validator::ValidationErrors, Valid};
use std::convert::TryFrom;
use uuid::Uuid;

type WithHardwareSpec = dsl::Eq<
    program_specs::columns::hardware_spec_id,
    Bound<sql_types::Uuid, Uuid>,
>;

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

impl ProgramSpec {
    /// Eq clause to compare the hardware_spec_id to a value.
    pub fn with_hardware_spec(hardware_spec_id: Uuid) -> WithHardwareSpec {
        program_specs::dsl::hardware_spec_id.eq(hardware_spec_id)
    }

    /// Start a query that filters by hardware spec ID.
    pub fn filter_by_hardware_spec(
        hardware_spec_id: Uuid,
    ) -> dsl::Filter<program_specs::table, WithHardwareSpec> {
        program_specs::table.filter(Self::with_hardware_spec(hardware_spec_id))
    }
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
#[derive(Clone, Debug, PartialEq, Insertable)]
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
