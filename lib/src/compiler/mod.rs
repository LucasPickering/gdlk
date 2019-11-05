//! This module contains all of the compiler steps. Each step is defined in its
//! own file, but they are all defined as functions on the
//! [`Compiler`](Compiler) struct.

mod codegen;
mod parse;
mod well_formed;

/// Struct to contain all compiler pipeline steps. By having this on a struct,
/// it makes it nice and easy to call functions in order with readability. Each
/// compiler step should take a `self` param and return a new `Compiler`.
///
/// `T` is the current type of the program. This controls which compiler
/// pipeline stages can be called. For example, if `T` is `File`, then
/// `.parse` is the only available operation. This allows us to leverage the
/// type system to enforce assumptions we might make in each compiler stage.
///
/// The value in the compiler is deliberately private, to prevent a compiler
/// from being directly constructed from outside this module. This means that
/// you must follow the proper pipeline stages to get the compiler to a certain
/// state.
pub struct Compiler<T>(T);

impl Compiler<()> {
    pub fn new() -> Self {
        Compiler(())
    }
}
