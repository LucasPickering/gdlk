use crate::{
    debug, error::CompileError, lang::ast::MachineInstr, models::Environment,
};
use std::fmt::Debug;

mod ast;
mod consts;
mod desugar;
mod machine;
mod parse;

pub use crate::lang::{
    ast::{LangValue, RegisterRef, StackIdentifier, UserRegisterIdentifier},
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
                num_registers: 1,
                num_stacks: 0,
                max_stack_length: 0,
                input: vec![1, 2],
                expected_output: vec![1, 2],
            },
            "
            READ RX0
            WRITE RX0
            READ RX0
            WRITE RX0
            ",
        );
    }

    #[test]
    fn test_set_push_pop() {
        execute_expect_success(
            Environment {
                id: 0,
                num_registers: 2,
                num_stacks: 1,
                max_stack_length: 5,
                input: vec![],
                expected_output: vec![10, 5],
            },
            "
            SET RX0 10
            PUSH RX0 S0
            SET RX0 0
            POP S0 RX0
            WRITE RX0
            SET RX1 5
            SET RX0 RX1
            WRITE RX0
            ",
        );
    }

    #[test]
    fn test_if() {
        execute_expect_success(
            Environment {
                id: 0,
                num_registers: 1,
                num_stacks: 0,
                max_stack_length: 0,
                input: vec![],
                expected_output: vec![1],
            },
            "
            IF RX0 {
                WRITE RX0
            }
            SET RX0 1
            IF RX0 {
                WRITE RX0
            }
            ",
        );
    }

    #[test]
    fn test_while() {
        execute_expect_success(
            Environment {
                id: 0,
                num_registers: 1,
                num_stacks: 1,
                max_stack_length: 5,
                input: vec![],
                expected_output: vec![2, 1, 0],
            },
            "
            PUSH RX0 S0
            SET RX0 1
            PUSH RX0 S0
            SET RX0 2
            PUSH RX0 S0
            WHILE RX0 {
                POP S0 RX0
                WRITE RX0
            }
            ",
        );
    }

    #[test]
    fn test_arithmetic() {
        execute_expect_success(
            Environment {
                id: 0,
                num_registers: 2,
                num_stacks: 0,
                max_stack_length: 0,
                input: vec![],
                expected_output: vec![-3, 140],
            },
            "
            ADD RX0 1
            SUB RX0 2
            MUL RX0 3
            WRITE RX0

            SET RX0 5
            SET RX1 10
            ADD RX0 RX1
            MUL RX0 RX1
            SUB RX0 RX1
            WRITE RX0
            ",
        );
    }

    #[test]
    fn test_square_all() {
        execute_expect_success(
            Environment {
                id: 0,
                num_registers: 1,
                num_stacks: 0,
                max_stack_length: 0,
                input: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                expected_output: vec![1, 4, 9, 16, 25, 36, 49, 64, 81, 100],
            },
            "
            WHILE RLI {
                READ RX0
                MUL RX0 RX0
                WRITE RX0
            }
            ",
        );
    }

    #[test]
    fn test_fibonacci() {
        execute_expect_success(
            Environment {
                id: 0,
                num_registers: 4,
                num_stacks: 0,
                max_stack_length: 0,
                input: vec![10],
                expected_output: vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34],
            },
            "
            READ RX0
            SET RX1 0
            SET RX2 1
            WHILE RX0 {
                WRITE RX1
                SET RX3 RX2
                ADD RX2 RX1
                SET RX1 RX3
                SUB RX0 1
            }
            ",
        );
    }
}
