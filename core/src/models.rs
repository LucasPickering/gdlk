//! This module holds general structs that don't fit anywhere else. These
//! structs should mostly just be data containers, with little to no
//! functionality defined on them.

#[cfg(feature = "wasm")]
use crate::ast::wasm::StringArray;
use crate::ast::{LangValue, RegisterRef, StackRef};
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use wasm_bindgen::{prelude::*, JsCast};

/// The "hardware" that a program can execute on. This defines computing
/// constraints. This is needed both at compile time and runtime.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HardwareSpec {
    // TODO make these readonly and camel case in wasm
    /// Number of registers available
    pub num_registers: usize,
    /// Maximum number of stacks permitted
    pub num_stacks: usize,
    /// Maximum size of each stack
    pub max_stack_length: usize,
}

// Functions that DON'T get exported to wasm
impl HardwareSpec {
    /// Get a list of all [RegisterRef]s that exist for this hardware.
    pub fn all_register_refs(&self) -> Vec<RegisterRef> {
        // The order here is important. This is how it will appear in the UI!
        // RLI first
        let mut register_refs = vec![RegisterRef::InputLength];
        // RSx registers
        register_refs
            .extend((0..self.num_stacks).map(RegisterRef::StackLength));
        // RXx registers
        register_refs.extend((0..self.num_registers).map(RegisterRef::User));
        register_refs
    }

    /// Get a list of all [StackRef]s that exist for this hardware.
    pub fn all_stack_refs(&self) -> Vec<StackRef> {
        (0..self.num_stacks).map(StackRef).collect()
    }
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

    /// A wrapper around [Self::all_register_refs] to be called from wasm.
    /// The elements of the returned vector are strings representing the name
    /// of each register.
    ///
    /// TODO change to `Vec<String>` after
    /// https://github.com/rustwasm/wasm-bindgen/issues/168
    #[cfg(feature = "wasm")]
    #[wasm_bindgen(getter, js_name = "registers")]
    pub fn wasm_registers(&self) -> StringArray {
        let refs = self.all_register_refs();
        let reg_names: Vec<String> = refs
            .into_iter()
            .map(|reg_ref| reg_ref.to_string())
            .collect();
        // Convert the vec to a js array. Be careful here!
        JsValue::from_serde(&reg_names).unwrap().unchecked_into()
    }

    /// A wrapper around [Self::all_stack_refs] to be called from wasm.
    /// The elements of the returned vector are strings representing the name
    /// of each stack.
    ///
    /// TODO change to `Vec<String>` after
    /// https://github.com/rustwasm/wasm-bindgen/issues/168
    #[cfg(feature = "wasm")]
    #[wasm_bindgen(getter, js_name = "stacks")]
    pub fn wasm_stacks(&self) -> StringArray {
        let stack_names: Vec<String> = self
            .all_stack_refs()
            .into_iter()
            .map(|stack_ref| stack_ref.to_string())
            .collect();
        // Convert the vec to a js array. Be careful here!
        JsValue::from_serde(&stack_names).unwrap().unchecked_into()
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
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProgramSpec {
    /// The input values, where the element at position 0 is the first one that
    /// will be popped off.
    input: Vec<LangValue>,
    /// The correct value to be left in the output when the program exits. The
    /// first element will be the first one pushed, and so on.
    expected_output: Vec<LangValue>,
}

// Functions that DON'T get exported to wasm
impl ProgramSpec {
    /// Get the program spec's defined input buffer. This is the initial value
    /// of the input for any run of the program.
    pub fn input(&self) -> &[LangValue] {
        &self.input
    }

    /// Get the program spec's expected output buffer. This is the output that
    /// a solution has to generate in order to be correct.
    pub fn expected_output(&self) -> &[LangValue] {
        &self.expected_output
    }
}

// Functions that get exported to wasm
#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl ProgramSpec {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new(input: Vec<LangValue>, expected_output: Vec<LangValue>) -> Self {
        ProgramSpec {
            input,
            expected_output,
        }
    }

    /// Version of [Self::input] to be called from wasm
    #[cfg(feature = "wasm")]
    #[wasm_bindgen(getter, js_name = "input")]
    pub fn wasm_input(&self) -> Vec<LangValue> {
        self.input.clone()
    }

    /// Version of [Self::expected_output] to be called from wasm
    #[cfg(feature = "wasm")]
    #[cfg_attr(
        feature = "wasm",
        wasm_bindgen(getter, js_name = "expectedOutput")
    )]
    pub fn wasm_expected_output(&self) -> Vec<LangValue> {
        self.expected_output.clone()
    }
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
                RegisterRef::StackLength(0),
                RegisterRef::StackLength(1),
                RegisterRef::User(0),
                RegisterRef::User(1),
                RegisterRef::User(2),
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
