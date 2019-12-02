use crate::{
    debug, error::CompileError, lang::ast::MachineInstr, models::Environment,
};
use std::fmt::Debug;

mod ast;
mod desugar;
mod machine;
mod parse;

pub use crate::lang::{
    ast::{LangValue, StackIdentifier},
    machine::{Machine, MachineState},
};

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
    /// Compiles a program into a [Machine](Machine). This takes an environment,
    /// which the program will executing in, and builds a machine around it so
    /// that it can be executed.
    pub fn compile(self, env: &Environment) -> Machine {
        Machine::new(env, self.0)
    }
}

/// Compiles the given source program, with the given environment, into a
/// [Machine](Machine). The returned machine can then be executed.
pub fn compile(
    env: &Environment,
    source: String,
) -> Result<Machine, CompileError> {
    Ok(Compiler::new(source)
        .debug()
        .parse()?
        .debug()
        .desugar()
        .debug()
        .compile(env))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn execute_expect_success(env: Environment, src: &str) {
        // Compile from env+src
        let mut machine = compile(&env, src.into()).unwrap();

        // Execute to completion
        let success = machine.execute_all().unwrap();

        // Make sure program terminated successfully
        // Check each bit of state individually to make debugging easier
        let state = machine.get_state();
        assert_eq!(state.input, Vec::new() as Vec<LangValue>);
        assert_eq!(state.output, env.expected_output);
        // Final sanity check, in case we change the criteria for success
        assert!(success);
    }

    #[test]
    fn test_read_write() {
        execute_expect_success(
            Environment {
                id: 0,
                num_stacks: 0,
                max_stack_size: None,
                input: vec![1, 2],
                expected_output: vec![1, 2],
            },
            "
            READ
            WRITE
            READ
            WRITE
            ",
        );
    }

    #[test]
    fn test_set_push_pop() {
        execute_expect_success(
            Environment {
                id: 0,
                num_stacks: 1,
                max_stack_size: None,
                input: vec![],
                expected_output: vec![10],
            },
            "
            SET 10
            PUSH 0
            SET 0
            POP 0
            WRITE
            ",
        );
    }

    #[test]
    fn test_if() {
        execute_expect_success(
            Environment {
                id: 0,
                num_stacks: 0,
                max_stack_size: None,
                input: vec![],
                expected_output: vec![1],
            },
            "
            IF {
                WRITE
            }
            SET 1
            IF {
                WRITE
            }
            ",
        );
    }

    #[test]
    fn test_while() {
        execute_expect_success(
            Environment {
                id: 0,
                num_stacks: 1,
                max_stack_size: None,
                input: vec![],
                expected_output: vec![2, 1, 0],
            },
            "
            PUSH 0
            SET 1
            PUSH 0
            SET 2
            PUSH 0
            WHILE {
                POP 0
                WRITE
            }
            ",
        );
    }
}
