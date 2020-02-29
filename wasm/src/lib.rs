// https://prestonrichey.com/blog/react-rust-wasm/
use gdlk::{Compiler, HardwareSpec, Valid};

use wasm_bindgen::{prelude::*, JsValue};
#[wasm_bindgen]
pub fn compile(source: &str, hw_spec: HardwareSpec) -> JsValue {
    let valid_spec = match Valid::validate(hw_spec) {
        Ok(spec) => spec,
        _ => return JsValue::from_str("ERROR: Invalid Hardware Spec"),
    };

    match Compiler::compile(source.to_string(), valid_spec) {
        Ok(res) => JsValue::from_serde(&res.program()).unwrap(),
        Err(err) => JsValue::from_serde(&err).unwrap(),
    }
}

#[wasm_bindgen]
pub fn make_hardware_spec(
    num_registers: usize,
    num_stacks: usize,
    max_stack_length: usize,
) -> HardwareSpec {
    HardwareSpec {
        num_registers: num_registers,
        num_stacks: num_stacks,
        max_stack_length: max_stack_length,
    }
}
