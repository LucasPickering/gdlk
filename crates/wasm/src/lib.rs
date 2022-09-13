#![deny(clippy::all)]
// https://prestonrichey.com/blog/react-rust-wasm/
use gdlk::ast::wasm::SourceElementArray;
pub use gdlk::{
    ast::{compiled::Program, wasm::SourceElement, LangValue},
    Compiler, HardwareSpec, Machine, ProgramSpec, Span,
};
use wasm_bindgen::{prelude::*, JsCast};

#[wasm_bindgen]
#[derive(Debug)]
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
            .map(|instr| {
                let span = instr.1;
                SourceElement {
                    // TODO use doc string here or something
                    text: "TODO".into(),
                    span,
                }
            })
            .collect();
        JsValue::from_serde(&instructions).unwrap().unchecked_into()
    }

    #[wasm_bindgen(getter)]
    pub fn machine(&self) -> Machine {
        self.machine.clone()
    }
}

/// Compile a program under the given specifications. This takes in references
/// so we don't have to move the values out of JS memory.
#[wasm_bindgen]
pub fn compile(
    hardware_spec: &HardwareSpec,
    program_spec: &ProgramSpec,
    source: &str,
) -> Result<CompileSuccess, JsValue> {
    match Compiler::compile(source.to_string(), *hardware_spec) {
        Ok(compiler) => {
            let program = compiler.program().clone();
            let machine = compiler.allocate(program_spec);
            Ok(CompileSuccess { program, machine })
        }
        Err(err) => {
            let errors: Vec<SourceElement> =
                err.errors().iter().map(SourceElement::from).collect();
            Err(JsValue::from_serde(&errors).unwrap())
        }
    }
}
