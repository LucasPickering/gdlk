//! Core implementation of the GDLK language. The main usage of this crate is
//! to compile and execute GDLK programs from source. A GDLK program runs under
//! a certain [HardwareSpec], which defines what resources it can access during
//! execution. It also runs against a [ProgramSpec], which defines the inputs
//! and expected outputs.
//!
//! ```
//! use gdlk::{HardwareSpec, ProgramSpec, Valid, Compiler};
//!
//! // Create the specs
//! let hardware_spec = Valid::validate(HardwareSpec {
//!     num_registers: 1,
//!     num_stacks: 0,
//!     max_stack_length: 0,
//! }).unwrap();
//! let program_spec = Valid::validate(ProgramSpec {
//!     input: vec![1],
//!     expected_output: vec![2],
//! }).unwrap();
//!
//! // Write your program
//! let source: String = "
//! READ RX0
//! ADD RX0 1
//! WRITE RX0
//! ".into();
//!
//! // Compile
//! let compiled = Compiler::compile(
//!     source,
//!     hardware_spec,
//! ).unwrap();
//!
//! // Execute
//! let mut machine = compiled.allocate(&program_spec);
//! machine.execute_all().unwrap();
//! assert!(machine.is_successful());
//! ```

#![deny(clippy::all, unused_must_use, unused_imports)]

#[macro_use]
extern crate validator_derive;

pub mod ast;
mod consts;
mod delabel;
pub mod error;
mod machine;
mod models;
mod parse;
mod util;
mod validate;

pub use consts::MAX_CYCLE_COUNT;
pub use machine::*;
pub use models::*;
pub use util::{Span, Valid};
pub use validator; // Consumers may need this

use ast::compiled::Program;
use error::{CompileError, WithSource};
use std::fmt::Debug;

/// Struct used to compile a program. `T` represents the current type of the
/// program. It starts as a [String], and as the compiler executes, the program
/// gets transformed. See the library-level documentation for examples on how to
/// compile and execute a program.
#[derive(Debug)]
pub struct Compiler<T: Debug> {
    // These are deliberately private, to prevent direct construction
    source: String,
    hardware_spec: Valid<HardwareSpec>,
    ast: T,
}

impl Compiler<()> {
    /// Compile a source program. Compiles under the constraints of a
    /// [HardwareSpec], which defines which registers and stacks are valid. The
    /// resulting compiled program can be used directly (e.g. for interactive
    /// syntax) or used to allocate a [Machine] that can be executed. See
    /// library-level documentation for more info.
    pub fn compile(
        source: String,
        hardware_spec: Valid<HardwareSpec>,
    ) -> Result<Compiler<Program<Span>>, WithSource<CompileError>> {
        Ok(Self {
            source,
            hardware_spec,
            ast: (),
        }
        .debug()
        .parse()?
        .debug()
        .validate()?
        .debug()
        .delabel()
        .debug())
    }
}

impl Compiler<Program<Span>> {
    /// Allocate a new [Machine] to execute a compiled program. The returned
    /// machine can then be executed. `program_spec` defines the parameters
    /// under which the program will execute.
    pub fn allocate(self, program_spec: &Valid<ProgramSpec>) -> Machine {
        Machine::new(&self.hardware_spec, program_spec, self.ast, self.source)
    }
}

impl<T: Debug> Compiler<T> {
    /// Print out the current state of this compiler, if debug mode is enabled.
    /// Takes in self and returns the same value, so that this can be used
    /// in the function call chain.
    fn debug(self) -> Self {
        debug!(println!("{:?}", &self));
        self
    }
}
