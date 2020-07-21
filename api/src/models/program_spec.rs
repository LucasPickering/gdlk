use crate::{
    models::{Factory, HardwareSpec},
    schema::program_specs,
};
use diesel::{
    dsl, expression::bound::Bound, prelude::*, query_builder::InsertStatement,
    sql_types, Identifiable, Queryable,
};
use uuid::Uuid;
use validator::Validate;

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
    /// URL-friendly identifier, unique to all program specs for a particular
    /// hardware spec (i.e. unique with `hardware_spec_id`). Derived from name.
    pub slug: String,
    /// User-readable name for this program spec.
    pub name: String,
    /// User-readable description of the problem.
    pub description: String,
    /// ID of the hardware that this program runs on
    pub hardware_spec_id: Uuid,
    /// The input values, where the element at position 0 is the first one that
    /// will be popped off.
    pub input: Vec<i32>,
    /// The correct value to be left in the output when the program exits. The
    /// first element will be the first one pushed, and so on.
    pub expected_output: Vec<i32>,
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

/// A derivative of [ProgramSpec](gdlk::ProgramSpec), meant for DB inserts.
/// This can be constructed manually and inserted into the DB. These fields
/// all correspond to [ProgramSpec](ProgramSpec), so look there for
/// field-level documentation.
#[derive(Clone, Debug, Default, PartialEq, Insertable, Validate)]
#[table_name = "program_specs"]
pub struct NewProgramSpec<'a> {
    pub hardware_spec_id: Uuid,
    #[validate(length(min = 1))]
    pub name: &'a str,
    pub description: &'a str,

    // IMPORTANT: If you update these validation values, make sure you update
    // ProgramSpec in the core crate as well!
    // TODO once we remove validator, change these to slices
    #[validate(length(max = 256))]
    pub input: Vec<i32>,
    #[validate(length(max = 256))]
    pub expected_output: Vec<i32>,
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

// This trait is only needed for tests
impl Factory for NewProgramSpec<'_> {
    type ReturnType = ProgramSpec;

    fn create(self, conn: &PgConnection) -> ProgramSpec {
        self.insert()
            .returning(program_specs::all_columns)
            .get_result(conn)
            .unwrap()
    }
}

#[derive(Clone, Debug, PartialEq, Identifiable, AsChangeset, Validate)]
#[table_name = "program_specs"]
pub struct ModifiedProgramSpec<'a> {
    pub id: Uuid,

    // TODO de-dupe this validation logic
    #[validate(length(min = 1))]
    pub name: Option<&'a str>,
    pub description: Option<&'a str>,
    // TODO once we remove validator, change these to slices
    #[validate(length(max = 256))]
    pub input: Option<Vec<i32>>,
    #[validate(length(max = 256))]
    pub expected_output: Option<Vec<i32>>,
}
