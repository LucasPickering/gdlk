//! Core implementation of the GDLK language. The main usage of this crate is
//! to compile and execute GDLK programs from source. A GDLK program runs under
//! a certain [HardwareSpec](HardwareSpec), which defines what resources it
//! can access during execution. It also runs against a
//! [ProgramSpec](ProgramSpec), which defines the inputs and expected outputs.
//!
//! ```
//! use gdlk::{HardwareSpec, ProgramSpec, Valid, compile_and_allocate};
//!
//! let hardware_spec = Valid::validate(HardwareSpec {
//!     num_registers: 1,
//!     num_stacks: 0,
//!     max_stack_length: 0,
//! }).unwrap();
//! let program_spec = Valid::validate(ProgramSpec {
//!     input: vec![1],
//!     expected_output: vec![2],
//! }).unwrap();
//! let source = "
//! READ RX0
//! ADD RX0 1
//! WRITE RX0
//! ";
//!
//! let mut machine = compile_and_allocate(
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
mod delabel;
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
pub use util::Valid;
pub use validator; // Consumers may need this

use ast::{compiled::Program, Span};
use std::fmt::Debug;

/// Compiles a source program, under the constraints of a hardware spec. Returns
/// the compiled program, or any errors that occurred.
pub fn compile(
    hardware_spec: &Valid<HardwareSpec>,
    source: &str,
) -> Result<Program<Span>, CompileErrors> {
    Ok(Compiler::new(source)
        .debug()
        .parse()?
        .debug()
        .validate(hardware_spec)?
        .debug()
        .delabel()
        .debug()
        .0)
}

/// Compiles a source program, under a hardware spec. Then, allocates a new
/// <Machine> to execute that program. The returned machine can then be
/// executed.
pub fn compile_and_allocate(
    hardware_spec: &Valid<HardwareSpec>,
    program_spec: &Valid<ProgramSpec>,
    source: &str,
) -> Result<Machine, CompileErrors> {
    Ok(Machine::new(
        hardware_spec,
        program_spec,
        compile(hardware_spec, source)?,
    ))
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

impl<'a> Compiler<&'a str> {
    /// Constructs a new compiler with no internal state. This is how you start
    /// a fresh compiler pipeline.
    pub fn new(source: &'a str) -> Self {
        Compiler(source)
    }
}
