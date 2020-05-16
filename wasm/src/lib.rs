#![deny(clippy::all, unused_must_use, unused_imports)]
// https://prestonrichey.com/blog/react-rust-wasm/
use gdlk::Valid;
pub use gdlk::{Compiler, HardwareSpec, Span};

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
