//! This module holds general structs that don't fit anywhere else. These
//! structs should mostly just be data containers, with little to no
//! functionality defined on them.

use crate::ast::LangValue;
use serde::{Deserialize, Serialize};

/// The "hardware" that a program can execute on. This defines computing
/// constraints. This is needed both at compile time and runtime.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct HardwareSpec {
    /// Number of registers available
    pub num_registers: usize,
    /// Maximum number of stacks permitted
    pub num_stacks: usize,
    /// Maximum size of each stack
    pub max_stack_length: usize,
}

// Useful for creating this type in tests
#[cfg(test)]
impl Default for HardwareSpec {
    fn default() -> Self {
        Self {
            num_registers: 1,
            num_stacks: 0,
            max_stack_length: 0,
        }
    }
}

/// Specification that defines a correct program. Provides the input that a
/// program runs on, and defines the expected output, which is used to determine
/// if the program is correct. Only needed at runtime.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProgramSpec {
    /// The input values, where the element at position 0 is the first one that
    /// will be popped off.
    pub input: Vec<LangValue>,
    /// The correct value to be left in the output when the program exits. The
    /// first element will be the first one pushed, and so on.
    pub expected_output: Vec<LangValue>,
}
