//! Integration tests for GDLK that expect compile errors. The programs in
//! these tests should all fail during execution.

use gdlk::{Compiler, HardwareSpec, ProgramSpec};

/// Compiles the program for the given hardware, executes it under the given
/// program spec, and expects a runtime error. Panics if the program executes
/// successfully, or if the wrong set of errors is returned.
macro_rules! assert_runtime_error {
    ($hw_spec:expr,$program_spec:expr, $src:expr, $expected_error:expr $(,)?) => {{
        // Compile from hardware+src
        let mut machine = Compiler::compile($src.into(), $hw_spec)
            .unwrap()
            .allocate(&($program_spec));

        // Execute to completion
        let actual_error = machine.execute_all().unwrap_err();
        assert_eq!(actual_error.to_string(), $expected_error);
        machine
    }};
}

#[test]
fn test_divide_by_zero() {
    assert_runtime_error!(
        HardwareSpec::default(),
        ProgramSpec::default(),
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
        ProgramSpec::default(),
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
        ProgramSpec::default(),
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
        ProgramSpec::default(),
        "POP S0 RX0",
        "Runtime error at 1:5: Cannot pop from empty stack `S0`",
    );
}

#[test]
fn test_exceed_max_cycle_count() {
    assert_runtime_error!(
        HardwareSpec::default(),
        ProgramSpec::default(),
        "
        START:
        JMP START
        ",
        "Runtime error at 3:9: Maximum number of cycles reached, \
            cannot execute instruction `JMP START`",
    );
}

#[test]
fn test_execute_after_error() {
    // Excuting after an error returns false
    let mut machine = assert_runtime_error!(
        HardwareSpec::default(),
        ProgramSpec::default(),
        "READ RX0",
        "Runtime error at 1:1: Read attempted while input is empty"
    );
    assert!(!machine.execute_next().unwrap());
}

#[test]
fn test_no_success_on_error() {
    // Make sure that if a program reaches a success state, but has further
    // instructions, then an error occurs, the execution is NOT considered
    // successful
    let machine = assert_runtime_error!(
        HardwareSpec::default(),
        ProgramSpec::new(vec![1], vec![1]),
        "
        READ RX0
        WRITE RX0
        ; if we were to exit here, it would be successful
        READ RX0 ; runtime error!
        ",
        "Runtime error at 5:9: Read attempted while input is empty"
    );

    assert!(!machine.successful());
}
