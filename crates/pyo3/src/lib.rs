use gdlk::{Compiler, HardwareSpec};
use pyo3::{exceptions::PyValueError, prelude::*};

#[pyfunction]
fn compile(source: &str) -> PyResult<String> {
    let hardware_spec = HardwareSpec {
        num_registers: 1,
        num_stacks: 0,
        max_stack_length: 16,
    };
    match Compiler::compile(source.into(), hardware_spec) {
        Ok(compiler) => Ok(format!("{:#?}", compiler.program())),
        Err(errors) => {
            // TODO better error formatting
            Err(PyValueError::new_err(format!("{:#?}", errors)))
        }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn gdlk_pyo3(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    Ok(())
}
