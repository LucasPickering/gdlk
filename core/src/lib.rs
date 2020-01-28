//! Core implementation of the GDLK language. The main usage of this crate is
//! to compile and execute GDLK programs from source. A GDLK program runs under
//! a certain [HardwareSpec](HardwareSpec), which defines what resources it
//! can access during execution. It also runs against a
//! [ProgramSpec](ProgramSpec), which defines the inputs and expected outputs.
//!
//! ```
//! use gdlk::{HardwareSpec, ProgramSpec, compile};
//!
//! let hardware_spec = HardwareSpec {
//!     num_registers: 1,
//!     num_stacks: 0,
//!     max_stack_length: 0,
//! };
//! let program_spec = ProgramSpec {
//!     input: vec![1],
//!     expected_output: vec![2],
//! };
//! let source = "
//! READ RX0
//! ADD RX0 1
//! WRITE RX0
//! ";
//!
//! let mut machine = compile(
//!     &hardware_spec,
//!     &program_spec,
//!     source.into(),
//! ).unwrap();
//! machine.execute_all().unwrap();
//! assert!(machine.is_successful());
//! ```

#![deny(clippy::all, unused_must_use, unused_imports)]
#![feature(try_trait)]

#[macro_use]
extern crate validator_derive;

pub mod ast;
mod consts;
mod desugar;
mod error;
mod machine;
mod models;
mod parse;
mod util;
mod validate;

pub use consts::MAX_CYCLE_COUNT;
pub use error::*;
pub use machine::*;
pub use models::*;

use ast::MachineInstr;
use std::fmt::Debug;
use validator::Validate;

/// Compiles the given source program, with the given specs, into a
/// [Machine](Machine). The returned machine can then be executed.
pub fn compile(
    hardware_spec: &HardwareSpec,
    program_spec: &ProgramSpec,
    source: String,
) -> Result<Machine, CompileErrors> {
    // Validate the specs, so we know that shit is clean
    hardware_spec
        .validate()
        .map_err(CompileError::InvalidSpec)?;
    program_spec.validate().map_err(CompileError::InvalidSpec)?;

    Ok(Compiler::new(source)
        .debug()
        .parse()?
        .debug()
        .validate(hardware_spec)?
        .debug()
        .desugar()
        .debug()
        .compile(hardware_spec, program_spec))
}

/// Struct to contain all compiler pipeline steps. By having this on a struct,
/// it makes it nice and easy to call functions in order with readability. Each
/// compiler step should take a `self` param and return a new `Compiler`.
///
/// `T` is the current type of the program. This controls which compiler
/// pipeline stages can be called. For example, if `T` is `()`, then
/// `.parse` is the only available operation. This allows us to leverage the
/// type system to enforce assumptions we might make in each compiler stage.
///
/// The value in the compiler is deliberately private, to prevent a compiler
/// from being directly constructed from outside this module. This means that
/// you must follow the proper pipeline stages to get the compiler to a certain
/// state.
#[derive(Debug)]
struct Compiler<T: Debug>(T);

impl<T: Debug> Compiler<T> {
    /// Prints out the current state of this compiler, if debug mode is enabled.
    /// Takes in self and returns the same value, so that this can be used
    /// in the function call chain.
    pub fn debug(self) -> Self {
        debug!(println!("{:?}", &self));
        self
    }
}

impl Compiler<String> {
    /// Constructs a new compiler with no internal state. This is how you start
    /// a fresh compiler pipeline.
    pub fn new(source: String) -> Self {
        Compiler(source)
    }
}

impl Compiler<Vec<MachineInstr>> {
    /// Compiles a program into a [Machine](Machine). This takes an hardware
    /// spec, which the program will execute on, and a program spec, which the
    /// program will try to match, and builds a machine around it so that it can
    /// be executed.
    pub fn compile(
        self,
        hardware_spec: &HardwareSpec,
        program_spec: &ProgramSpec,
    ) -> Machine {
        Machine::new(hardware_spec, program_spec, self.0)
    }
}
