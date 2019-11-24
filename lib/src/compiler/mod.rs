use crate::{
    ast::{Environment, MachineInstr},
    machine::Machine,
};

mod delabel;
mod desugar;
mod parse;

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
pub struct Compiler<T>(T);

impl Compiler<()> {
    /// Constructs a new compiler with no internal state. This is how you start
    /// a fresh compiler pipeline.
    pub fn new() -> Self {
        Compiler(())
    }
}

impl Compiler<Vec<MachineInstr>> {
    /// Compiles a program into a [Machine](Machine). This takes an environment,
    /// which the program will executing in, and builds a machine around it so
    /// that it can be executed.
    pub fn compile(self, env: Environment) -> Machine {
        Machine::new(env, self.0)
    }
}
