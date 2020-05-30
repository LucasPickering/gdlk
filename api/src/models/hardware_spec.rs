use crate::{models::Factory, schema::hardware_specs};
use diesel::{
    prelude::*, query_builder::InsertStatement, Identifiable, Queryable,
};
use gdlk::validator::Validate;
use uuid::Uuid;

/// A derivative of [HardwareSpec](gdlk::HardwareSpec), containing all fields.
/// that are present on the DB table. This should only ever be constructed from
/// a DB query.
#[derive(Clone, Debug, PartialEq, Identifiable, Queryable)]
#[table_name = "hardware_specs"]
pub struct HardwareSpec {
    /// DB row ID
    pub id: Uuid,
    /// Unique identifier that can be used in URLs. Derived from the name.
    pub slug: String,
    /// User-friendly name for this hardware.
    pub name: String,

    // These three need to be i32s because postgres has no unsigned type.
    // The insertion code and DB should both enforce that they are >= 0.
    /// Number of registers available
    pub num_registers: i32,
    /// Maximum number of stacks permitted
    pub num_stacks: i32,
    /// Maximum size of each stack
    pub max_stack_length: i32,
}

/// A derivative of [HardwareSpec](gdlk::HardwareSpec), meant for DB inserts.
/// This can be constructed manually and inserted into the DB. These fields
/// all correspond to [HardwareSpec](HardwareSpec), so look there for
/// field-level documentation.
#[derive(Debug, PartialEq, Insertable, Validate)]
#[table_name = "hardware_specs"]
pub struct NewHardwareSpec<'a> {
    #[validate(length(min = 1))]
    pub name: &'a str,

    // IMPORTANT: If you change any of the range values here, update
    // HardwareSpec in the core crate as well
    #[validate(range(min = 1, max = 16))]
    pub num_registers: i32,
    #[validate(range(min = 0, max = 16))]
    pub num_stacks: i32,
    #[validate(range(min = 0, max = 256))]
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

/// A struct used to modify a row in the hardware_specs table.
#[derive(Clone, Debug, PartialEq, Identifiable, AsChangeset, Validate)]
#[table_name = "hardware_specs"]
pub struct ModifiedHardwareSpec<'a> {
    pub id: Uuid,

    // TODO de-dupe this validation logic
    #[validate(length(min = 1))]
    pub name: Option<&'a str>,
    #[validate(range(min = 1, max = 16))]
    pub num_registers: Option<i32>,
    #[validate(range(min = 0, max = 16))]
    pub num_stacks: Option<i32>,
    #[validate(range(min = 0, max = 256))]
    pub max_stack_length: Option<i32>,
}
