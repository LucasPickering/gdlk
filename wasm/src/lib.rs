#![deny(clippy::all, unused_must_use, unused_imports)]
// https://prestonrichey.com/blog/react-rust-wasm/
use gdlk::{ast::wasm::SourceElementArray, Valid};
pub use gdlk::{
    ast::{compiled::Program, wasm::SourceElement},
    Compiler, HardwareSpec, Machine, ProgramSpec, Span,
};
use wasm_bindgen::{prelude::*, JsCast};

#[wasm_bindgen]
pub struct CompileSuccess {
    program: Program<Span>,
    machine: Machine,
}

#[wasm_bindgen]
impl CompileSuccess {
    /// Get the compiled instructions that make up the program. The instructions
    /// can be mapped back to their source.
    ///
    /// TODO change to `Vec<SourceElement>` after
    /// https://github.com/rustwasm/wasm-bindgen/issues/111
    #[wasm_bindgen(getter)]
    pub fn instructions(&self) -> SourceElementArray {
        // Convert the AST into a JS array of instruction descriptors
        let instructions: Vec<SourceElement> = self
            .program
            .instructions
            .iter()
            .map(|instr| SourceElement {
                // TODO use doc string here or something
                text: format!("{:?}", instr),
                span: instr.1,
            })
            .collect();
        JsValue::from_serde(&instructions).unwrap().unchecked_into()
    }

    #[wasm_bindgen(getter)]
    pub fn machine(&self) -> Machine {
        self.machine.clone()
    }
}

#[wasm_bindgen]
pub fn compile(
    hardware_spec: &HardwareSpec,
    program_spec: &ProgramSpec,
    source: &str,
) -> Result<CompileSuccess, JsValue> {
    // It _shouldn't_ be possible for this validation to fail since the specs
    // always comes from the DB, but if it does, we'll just blow up the page
    // We take in references so that the JS values don't get moved.
    let valid_hardware_spec = Valid::validate(*hardware_spec).unwrap();
    let valid_program_spec: Valid<&ProgramSpec> =
        Valid::validate(program_spec).unwrap();

    match Compiler::compile(source.to_string(), valid_hardware_spec) {
        Ok(compiler) => {
            let program = compiler.program().clone();
            let machine = compiler.allocate(valid_program_spec);
            Ok(CompileSuccess { program, machine })
        }
        Err(err) => {
            let errors: Vec<SourceElement> =
                err.errors().iter().map(SourceElement::from).collect();
            Err(JsValue::from_serde(&errors).unwrap())
        }
    }
}
