use crate::schema::{hardware_specs, program_specs};
use diesel::{Identifiable, Queryable};
use gdlk::{ast::LangValue, HardwareSpec, ProgramSpec};

/// A derivative of [HardwareSpec](gdlk::HardwareSpec), built from a DB query.
#[derive(Debug, PartialEq, Identifiable, Queryable)]
#[table_name = "hardware_specs"]
pub struct QueryHardwareSpec {
    /// DB row ID
    pub id: i32,

    // These three need to be i32s because postgres has no unsigned type.
    // The insertion code and DB should both enforce that they are >= 0.
    /// Number of registers available
    pub num_registers: i32,
    /// Maximum number of stacks permitted
    pub num_stacks: i32,
    /// Maximum size of each stack
    pub max_stack_length: i32,
}

impl From<QueryHardwareSpec> for HardwareSpec {
    fn from(other: QueryHardwareSpec) -> Self {
        Self {
            // TODO make these conversions safe
            num_registers: other.num_registers as usize,
            num_stacks: other.num_stacks as usize,
            max_stack_length: other.max_stack_length as usize,
        }
    }
}

/// A derivative of [ProgramSpec](gdlk::ProgramSpec), built from a DB query.
#[derive(Debug, PartialEq, Identifiable, Associations, Queryable)]
#[belongs_to(QueryHardwareSpec, foreign_key = "hardware_spec_id")]
#[table_name = "program_specs"]
pub struct QueryProgramSpec {
    /// DB row ID
    pub id: i32,

    /// ID of the hardware that this program runs on
    pub hardware_spec_id: i32,
    /// The input values, where the element at position 0 is the first one that
    /// will be popped off.
    pub input: Vec<LangValue>,
    /// The correct value to be left in the output when the program exits. The
    /// first element will be the first one pushed, and so on.
    pub expected_output: Vec<LangValue>,
}

impl From<QueryProgramSpec> for ProgramSpec {
    fn from(other: QueryProgramSpec) -> Self {
        Self {
            input: other.input,
            expected_output: other.expected_output,
        }
    }
}
