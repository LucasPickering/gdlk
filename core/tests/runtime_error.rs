//! Integration tests for GDLK that expect compile errors. The programs in
//! these tests should all fail during execution.

use gdlk::{Compiler, HardwareSpec, ProgramSpec, Valid};

/// Compiles the program for the given hardware, executes it under the given
/// program spec, and expects a runtime error. Panics if the program executes
/// successfully, or if the wrong set of errors is returned.
macro_rules! assert_runtime_error {
    ($hw_spec:expr,$program_spec:expr, $src:expr, $expected_error:expr $(,)?) => {
        // Compile from hardware+src
        let mut machine =
            Compiler::compile($src.into(), Valid::validate($hw_spec).unwrap())
                .unwrap()
                .allocate(Valid::validate($program_spec).unwrap());

        // Execute to completion
        let actual_error = machine.execute_all().unwrap_err();
        assert_eq!(format!("{}", actual_error), $expected_error);
    };
}

#[test]
fn test_divide_by_zero() {
    assert_runtime_error!(
        HardwareSpec::default(),
        &ProgramSpec::default(),
        "
        SET RX0 1
        DIV RX0 0
        ",
        "Runtime error at 3:9: Divide by zero",
    );
}

#[test]
fn test_stack_overflow() {
    assert_runtime_error!(
        HardwareSpec {
            num_registers: 1,
            num_stacks: 1,
            max_stack_length: 3,
        },
        &ProgramSpec::default(),
        "
        SET RX0 4
        START:
        PUSH RX0 S0
        SUB RX0 1
        JGZ RX0 START
        ",
        "Runtime error at 4:18: Overflow on stack `S0`",
    );
}

#[test]
fn test_empty_input() {
    assert_runtime_error!(
        HardwareSpec::default(),
        &ProgramSpec::default(),
        "READ RX0",
        "Runtime error at 1:1: Read attempted while input is empty",
    );
}

#[test]
fn test_empty_stack() {
    assert_runtime_error!(
        HardwareSpec {
            num_registers: 1,
            num_stacks: 1,
            max_stack_length: 3,
        },
        &ProgramSpec::default(),
        "POP S0 RX0",
        "Runtime error at 1:5: Cannot pop from empty stack `S0`",
    );
}

#[test]
fn test_exceed_max_cycle_count() {
    assert_runtime_error!(
        HardwareSpec::default(),
        &ProgramSpec::default(),
        "
        START:
        JMP START
        ",
        "Runtime error at 3:9: Maximum number of cycles reached, \
            cannot execute instruction `JMP START`",
    );
}
