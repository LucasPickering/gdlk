use crate::{
    debug,
    error::CompileErrors,
    lang::ast::MachineInstr,
    models::{HardwareSpec, ProgramSpec},
};
use std::fmt::Debug;

mod ast;
mod consts;
mod desugar;
mod machine;
mod parse;
mod validate;

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

/// Compiles the given source program, with the given environment, into a
/// [Machine](Machine). The returned machine can then be executed.
pub fn compile(
    hardware_spec: &HardwareSpec,
    program_spec: &ProgramSpec,
    source: String,
) -> Result<Machine, CompileErrors> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProgramSpec;

    /// Compiles the program for the given hardware, and executes it against the
    /// program spec.. Panics if the compile fails or the execution isn't
    /// successful.
    fn execute_expect_success(
        hardware_spec: HardwareSpec,
        program_spec: ProgramSpec,
        src: &str,
    ) {
        // Compile from hardware+src
        let mut machine =
            compile(&hardware_spec, &program_spec, src.into()).unwrap();

        // Execute to completion
        let success = machine.execute_all().unwrap();

        // Make sure program terminated successfully
        // Check each bit of state individually to make debugging easier
        let state = machine.get_state();
        assert_eq!(state.input, Vec::new() as Vec<i32>);
        assert_eq!(state.output, program_spec.expected_output);
        // Final sanity check, in case we change the criteria for success
        assert!(success);
    }

    /// Compiles the program for the given hardware, expecting compile error(s).
    /// Panics if the program compiles successfully, or if the wrong set of
    /// errors is returned.
    fn expect_compile_errors(
        env: HardwareSpec,
        src: &str,
        expected_errors: &[&str],
    ) {
        // Compile from hardware+src, use a default program spec because we
        // won't be executing this
        let actual_errors =
            compile(&env, &ProgramSpec::default(), src.into()).unwrap_err();
        assert_eq!(format!("{}", actual_errors), expected_errors.join("\n"));
    }

    #[test]
    fn test_read_write() {
        execute_expect_success(
            HardwareSpec::default(),
            ProgramSpec {
                input: vec![1, 2],
                expected_output: vec![1, 2],
                ..ProgramSpec::default()
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
            HardwareSpec {
                num_registers: 2,
                num_stacks: 1,
                max_stack_length: 5,
                ..HardwareSpec::default()
            },
            ProgramSpec {
                input: vec![],
                expected_output: vec![10, 5],
                ..ProgramSpec::default()
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
            HardwareSpec::default(),
            ProgramSpec {
                input: vec![],
                expected_output: vec![1],
                ..ProgramSpec::default()
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
            HardwareSpec {
                num_stacks: 1,
                max_stack_length: 5,
                ..HardwareSpec::default()
            },
            ProgramSpec {
                input: vec![],
                expected_output: vec![2, 1, 0],
                ..ProgramSpec::default()
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
            HardwareSpec {
                num_registers: 2,
                ..HardwareSpec::default()
            },
            ProgramSpec {
                input: vec![],
                expected_output: vec![-3, 140],
                ..ProgramSpec::default()
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
            HardwareSpec::default(),
            ProgramSpec {
                input: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                expected_output: vec![1, 4, 9, 16, 25, 36, 49, 64, 81, 100],
                ..ProgramSpec::default()
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
            HardwareSpec {
                num_registers: 4,
                ..HardwareSpec::default()
            },
            ProgramSpec {
                input: vec![10],
                expected_output: vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34],
                ..ProgramSpec::default()
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

    #[test]
    fn test_parse_empty_file() {
        expect_compile_errors(
            HardwareSpec::default(),
            "",
            &["Parse error: 0: in Alpha, got empty input\n\n"],
        );
    }

    #[test]
    fn test_invalid_user_reg_ref() {
        expect_compile_errors(
            HardwareSpec {
                num_registers: 1,
                num_stacks: 1,
                ..HardwareSpec::default()
            },
            "
            READ RX1
            WRITE RX2
            SET RX3 RX0
            ADD RX4 RX0
            SUB RX5 RX0
            MUL RX6 RX0
            PUSH RX7 S0
            POP S0 RX8
            ",
            &[
                "Invalid reference to register RX1",
                "Invalid reference to register RX2",
                "Invalid reference to register RX3",
                "Invalid reference to register RX4",
                "Invalid reference to register RX5",
                "Invalid reference to register RX6",
                "Invalid reference to register RX7",
                "Invalid reference to register RX8",
            ],
        );
    }

    #[test]
    fn test_invalid_stack_reg_ref() {
        expect_compile_errors(
            HardwareSpec {
                num_stacks: 1,
                ..HardwareSpec::default()
            },
            "
        SET RX0 RS1
        ",
            &["Invalid reference to register RS1"],
        );
    }

    #[test]
    fn test_invalid_stack_ref() {
        expect_compile_errors(
            HardwareSpec {
                num_stacks: 1,
                ..HardwareSpec::default()
            },
            "
            PUSH 5 S1
            POP S2 RX0
            ",
            &[
                "Invalid reference to stack S1",
                "Invalid reference to stack S2",
            ],
        );
    }

    #[test]
    fn test_unwritable_reg() {
        expect_compile_errors(
            HardwareSpec {
                num_stacks: 1,
                ..HardwareSpec::default()
            },
            "
            SET RLI 5
            SET RS0 5
            ",
            &[
                "Cannot write to read-only register RLI",
                "Cannot write to read-only register RS0",
            ],
        );
    }
}
