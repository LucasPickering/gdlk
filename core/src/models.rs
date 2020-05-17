//! This module holds general structs that don't fit anywhere else. These
//! structs should mostly just be data containers, with little to no
//! functionality defined on them.

use crate::ast::{LangValue, RegisterRef, StackRef};
use serde::{Deserialize, Serialize};
use validator::Validate;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
/// The "hardware" that a program can execute on. This defines computing
/// constraints. This is needed both at compile time and runtime.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Validate)]
pub struct HardwareSpec {
    /// Number of registers available
    #[validate(range(min = 1, max = 16))]
    pub num_registers: usize,
    /// Maximum number of stacks permitted
    #[validate(range(min = 0, max = 16))]
    pub num_stacks: usize,
    /// Maximum size of each stack
    #[validate(range(min = 0, max = 256))]
    pub max_stack_length: usize,
}

// Functions that get exported to wasm
#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl HardwareSpec {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new(
        num_registers: usize,
        num_stacks: usize,
        max_stack_length: usize,
    ) -> Self {
        HardwareSpec {
            num_registers,
            num_stacks,
            max_stack_length,
        }
    }
}

// Functions that DON'T get exported to wasm
impl HardwareSpec {
    /// Get a list of all [RegisterRef]s that exist for this hardware.
    pub fn all_register_refs(&self) -> Vec<RegisterRef> {
        let mut register_refs = vec![RegisterRef::InputLength];
        register_refs.extend((0..self.num_registers).map(RegisterRef::User));
        register_refs
            .extend((0..self.num_stacks).map(RegisterRef::StackLength));
        register_refs
    }

    /// Get a list of all [StackRef]s that exist for this hardware.
    pub fn all_stack_refs(&self) -> Vec<StackRef> {
        (0..self.num_stacks).map(StackRef).collect()
    }
}

// Useful for tests and prototyping
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Validate)]
pub struct ProgramSpec {
    /// The input values, where the element at position 0 is the first one that
    /// will be popped off.
    #[validate(length(max = 256))]
    pub input: Vec<LangValue>,
    /// The correct value to be left in the output when the program exits. The
    /// first element will be the first one pushed, and so on.
    #[validate(length(max = 256))]
    pub expected_output: Vec<LangValue>,
}

// Useful for tests and prototyping
impl Default for ProgramSpec {
    fn default() -> Self {
        Self {
            input: vec![],
            expected_output: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_register_refs() {
        assert_eq!(
            HardwareSpec {
                num_registers: 0,
                num_stacks: 0,
                max_stack_length: 0,
            }
            .all_register_refs(),
            vec![RegisterRef::InputLength],
        );

        assert_eq!(
            HardwareSpec {
                num_registers: 3,
                num_stacks: 2,
                max_stack_length: 0,
            }
            .all_register_refs(),
            vec![
                RegisterRef::InputLength,
                RegisterRef::User(0),
                RegisterRef::User(1),
                RegisterRef::User(2),
                RegisterRef::StackLength(0),
                RegisterRef::StackLength(1),
            ],
        );
    }

    #[test]
    fn test_all_stack_refs() {
        assert_eq!(
            HardwareSpec {
                num_registers: 0,
                num_stacks: 0,
                max_stack_length: 0,
            }
            .all_stack_refs(),
            vec![],
        );

        assert_eq!(
            HardwareSpec {
                num_registers: 3,
                num_stacks: 2,
                max_stack_length: 0,
            }
            .all_stack_refs(),
            vec![StackRef(0), StackRef(1),],
        );
    }
}
