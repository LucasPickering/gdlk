use crate::{
    lang::LangValue,
    schema::{hardware_specs, program_specs},
};
use diesel::{Identifiable, Queryable};
use serde::{Deserialize, Serialize};

/// The "hardware" that a program can execute on. This defines computing
/// constraints. This is needed both at compile time and runtime.
#[derive(Debug, PartialEq, Serialize, Deserialize, Identifiable, Queryable)]
pub struct HardwareSpec {
    /// DB row ID
    #[serde(default)]
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

// Useful for creating this type in tests
#[cfg(test)]
impl Default for HardwareSpec {
    fn default() -> Self {
        Self {
            id: 0,
            num_registers: 1,
            num_stacks: 0,
            max_stack_length: 0,
        }
    }
}

/// Specification that defines a correct program. Provides the input that a
/// program runs on, and defines the expected output, which is used to determine
/// if the program is correct. Only needed at runtime.
#[derive(
    Debug,
    PartialEq,
    Serialize,
    Deserialize,
    Identifiable,
    Associations,
    Queryable,
)]
#[belongs_to(HardwareSpec)]
pub struct ProgramSpec {
    /// DB row ID
    #[serde(default)]
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

// Useful for creating this type in tests
#[cfg(test)]
impl Default for ProgramSpec {
    fn default() -> Self {
        Self {
            id: 0,
            hardware_spec_id: 0,
            input: vec![],
            expected_output: vec![],
        }
    }
}
