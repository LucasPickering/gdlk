//! Integration tests for GDLK that expect program success. The programs in
//! these tests should compile successful, and execute with a successful
//! outcome.

use gdlk::{ast::LangValue, compile, HardwareSpec, ProgramSpec};

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
    assert_eq!(state.input, Vec::new() as Vec<LangValue>);
    assert_eq!(state.output, program_spec.expected_output);
    // Final sanity check, in case we change the criteria for success
    assert!(success);
}

#[test]
fn test_read_write() {
    execute_expect_success(
        HardwareSpec {
            num_registers: 1,
            num_stacks: 0,
            max_stack_length: 0,
        },
        ProgramSpec {
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
        HardwareSpec {
            num_registers: 2,
            num_stacks: 1,
            max_stack_length: 5,
        },
        ProgramSpec {
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
        HardwareSpec {
            num_registers: 1,
            num_stacks: 0,
            max_stack_length: 0,
        },
        ProgramSpec {
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
        HardwareSpec {
            num_registers: 1,
            num_stacks: 1,
            max_stack_length: 5,
        },
        ProgramSpec {
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
        HardwareSpec {
            num_registers: 2,
            num_stacks: 0,
            max_stack_length: 0,
        },
        ProgramSpec {
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
        HardwareSpec {
            num_registers: 1,
            num_stacks: 0,
            max_stack_length: 0,
        },
        ProgramSpec {
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
        HardwareSpec {
            num_registers: 4,
            num_stacks: 0,
            max_stack_length: 0,
        },
        ProgramSpec {
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
