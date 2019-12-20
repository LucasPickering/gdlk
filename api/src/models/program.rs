use crate::{
    models::hardware::FullHardwareSpec,
    schema::{hardware_specs, program_specs},
};
use diesel::{
    dsl, expression::bound::Bound, prelude::*, query_builder::InsertStatement,
    sql_types::Text, Identifiable, Queryable,
};
use gdlk::{ast::LangValue, ProgramSpec};
use std::convert::TryFrom;
use validator::{Validate, ValidationErrors};

type WithSlug<'a> = dsl::Eq<program_specs::columns::slug, Bound<Text, &'a str>>;

type WithHwSlug<'a> =
    dsl::Eq<hardware_specs::columns::slug, Bound<Text, &'a str>>;

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

impl FullProgramSpec {
    /// Filters program specs by their associated hardware spec's slug.
    pub fn filter_by_hw_slug<'a>(
        hw_spec_slug: &'a str,
    ) -> dsl::Filter<
        dsl::InnerJoin<program_specs::table, hardware_specs::table>,
        WithHwSlug<'a>,
    > {
        program_specs::dsl::program_specs
            .inner_join(hardware_specs::dsl::hardware_specs)
            .filter(hardware_specs::dsl::slug.eq(hw_spec_slug))
    }

    /// Filters program specs by their own slug and their associated hardware
    /// spec's slug. These two values together form a unique pair, so this
    /// query will always refer to 0 or 1 rows.
    pub fn filter_by_slugs<'a>(
        hw_spec_slug: &'a str,
        program_spec_slug: &'a str,
    ) -> dsl::Filter<
        dsl::InnerJoin<program_specs::table, hardware_specs::table>,
        dsl::And<WithHwSlug<'a>, WithSlug<'a>>,
    > {
        Self::filter_by_hw_slug(hw_spec_slug)
            .filter(program_specs::dsl::slug.eq(program_spec_slug))
    }
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
