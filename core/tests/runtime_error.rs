//! Integration tests for GDLK that expect compile errors. The programs in
//! these tests should all fail during compilation.

use gdlk::{compile, HardwareSpec, ProgramSpec};

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
        compile(&hardware_spec, &program_spec, src.into()).unwrap();

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
        "Overflow on stack S0",
    );
}

#[test]
fn test_empty_input() {
    expect_runtime_error(
        HardwareSpec::default(),
        ProgramSpec::default(),
        "READ RX0",
        "No input available to read",
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
        "Cannot pop from empty stack S0",
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
        "The maximum number of cycles has been reached",
    );
}
