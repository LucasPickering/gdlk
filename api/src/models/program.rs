use crate::{
    models::{hardware, HardwareSpec, User},
    schema::{hardware_specs, program_specs, user_programs, users},
};
use diesel::{
    dsl,
    expression::bound::Bound,
    prelude::*,
    query_builder::InsertStatement,
    sql_types::{Int4, Text},
    Identifiable, Queryable,
};
use gdlk::{ast::LangValue, validator::ValidationErrors, Valid};
use std::convert::TryFrom;

/// Inner join between program_specs and hardware_specs
type InnerJoinSpecs =
    dsl::InnerJoin<program_specs::table, hardware_specs::table>;

/// Expression to filter program_specs by slug
type WithSlug<'a> = dsl::Eq<program_specs::columns::slug, Bound<Text, &'a str>>;

/// Expression to filter program_specs by hardware spec slug and its own slug
type WithSlugs<'a> = dsl::And<hardware::WithSlug<'a>, WithSlug<'a>>;

/// Expression to filter user_programs by user ID, hardware spec slug, and
/// program spec slug
type WithUserAndSpecs<'a> =
    dsl::And<WithSlugs<'a>, dsl::Eq<users::columns::id, Bound<Int4, i32>>>;

/// Expression to filter user_programs by user ID, hardware spec slug,
/// program spec slug, and file name
type WithFileName<'a> = dsl::And<
    WithUserAndSpecs<'a>,
    dsl::Eq<user_programs::columns::file_name, Bound<Text, &'a str>>,
>;

/// A derivative of [ProgramSpec](gdlk::ProgramSpec), built from a DB query.
#[derive(Debug, PartialEq, Identifiable, Associations, Queryable)]
#[belongs_to(HardwareSpec, foreign_key = "hardware_spec_id")]
#[table_name = "program_specs"]
pub struct ProgramSpec {
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

impl ProgramSpec {
    /// Filters program specs by their associated hardware spec's slug.
    pub fn filter_by_hw_slug<'a>(
        hw_spec_slug: &'a str,
    ) -> dsl::Filter<InnerJoinSpecs, hardware::WithSlug<'a>> {
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
    ) -> dsl::Filter<InnerJoinSpecs, WithSlugs<'a>> {
        Self::filter_by_hw_slug(hw_spec_slug)
            .filter(program_specs::dsl::slug.eq(program_spec_slug))
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
#[derive(Debug, PartialEq, Insertable)]
#[table_name = "program_specs"]
pub struct NewProgramSpec<'a> {
    pub slug: &'a str,
    pub hardware_spec_id: i32,
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
    pub id: i32,
    pub user_id: i32,
    pub program_spec_id: i32,
    pub file_name: String,
    pub source_code: String,
}

impl UserProgram {
    pub fn with_file_name<'a>(
        file_name: &'a str,
    ) -> dsl::Eq<user_programs::columns::file_name, Bound<Text, &'a str>> {
        user_programs::dsl::file_name.eq(file_name)
    }

    /// Get all user programs that exist for a user-program spec pair.
    pub fn filter_by_specs<'a>(
        user_id: i32,
        hw_spec_slug: &'a str,
        program_spec_slug: &'a str,
    ) -> dsl::Filter<
        dsl::InnerJoin<
            InnerJoinSpecs,
            dsl::InnerJoin<user_programs::table, users::table>,
        >,
        WithUserAndSpecs<'a>,
    > {
        ProgramSpec::filter_by_slugs(hw_spec_slug, program_spec_slug)
            .inner_join(
                user_programs::dsl::user_programs.inner_join(users::dsl::users),
            )
            .filter(users::dsl::id.eq(user_id))
    }

    /// Filter down to exactly one UserProgram, based on a user, hw spec,
    /// program spec, and file name.
    pub fn filter_by_file_name<'a>(
        user_id: i32,
        hw_spec_slug: &'a str,
        program_spec_slug: &'a str,
        file_name: &'a str,
    ) -> dsl::Filter<
        dsl::InnerJoin<
            InnerJoinSpecs,
            dsl::InnerJoin<user_programs::table, users::table>,
        >,
        WithFileName<'a>,
    > {
        ProgramSpec::filter_by_slugs(hw_spec_slug, program_spec_slug)
            .inner_join(
                user_programs::dsl::user_programs.inner_join(users::dsl::users),
            )
            .filter(users::dsl::id.eq(user_id))
            .filter(Self::with_file_name(file_name))
    }
}

#[derive(Debug, PartialEq, Insertable)]
#[table_name = "user_programs"]
pub struct NewUserProgram<'a> {
    pub user_id: i32,
    pub program_spec_id: i32,
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
