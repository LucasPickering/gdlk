//! Integration tests for GDLK that expect compile errors. The programs in
//! these tests should all fail during compilation.

use gdlk::{Compiler, HardwareSpec, ProgramSpec, Valid};

/// Compiles the program for the given hardware, executes it under the given
/// program spec, and expects a runtime error. Panics if the program executes
/// successfully, or if the wrong set of errors is returned.
fn expect_runtime_error(
    hardware_spec: HardwareSpec,
    program_spec: ProgramSpec,
    src: &str,
    expected_error: &str,
) {
    // Compile from hardware+src
    let mut machine =
        Compiler::compile(src.into(), Valid::validate(hardware_spec).unwrap())
            .unwrap()
            .allocate(&Valid::validate(program_spec).unwrap());

    // Execute to completion
    let actual_error = machine.execute_all().unwrap_err();
    assert_eq!(format!("{}", actual_error), expected_error);
}

#[test]
fn test_stack_overflow() {
    expect_runtime_error(
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
        "Error on line 4: Overflow on stack `S0`",
    );
}

#[test]
fn test_empty_input() {
    expect_runtime_error(
        HardwareSpec::default(),
        ProgramSpec::default(),
        "READ RX0",
        "Error on line 1: Read attempted while input is empty",
    );
}

#[test]
fn test_empty_stack() {
    expect_runtime_error(
        HardwareSpec {
            num_registers: 1,
            num_stacks: 1,
            max_stack_length: 3,
        },
        ProgramSpec::default(),
        "POP S0 RX0",
        "Error on line 1: Cannot pop from empty stack `S0`",
    );
}

#[test]
fn test_exceed_max_cycle_count() {
    expect_runtime_error(
        HardwareSpec::default(),
        ProgramSpec {
            input: vec![],
            expected_output: vec![],
        },
        "
        START:
        JMP START
        ",
        "Error on line 3: Maximum number of cycles reached, \
        cannot execute instruction `JMP START`",
    );
}
