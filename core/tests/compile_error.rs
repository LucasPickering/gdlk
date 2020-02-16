//! Integration tests for GDLK that expect compile errors. The programs in
//! these tests should all fail during compilation.

use gdlk::{compile, HardwareSpec};

/// Compiles the program for the given hardware, expecting compile error(s).
/// Panics if the program compiles successfully, or if the wrong set of
/// errors is returned.
fn expect_compile_errors(
    hardware_spec: HardwareSpec,
    src: &str,
    expected_errors: &[&str],
) {
    // Compile from hardware+src
    let actual_errors = compile(&hardware_spec, src).unwrap_err();
    assert_eq!(format!("{}", actual_errors), expected_errors.join("\n"));
}

#[test]
fn test_parse_no_newline_after_inst() {
    // TODO: make this error nicer
    expect_compile_errors(
        HardwareSpec {
            num_registers: 1,
            num_stacks: 0,
            max_stack_length: 0,
        },
        "READ RX1 WRITE RX2",
        &["Parse error: Invalid keyword: WRITE RX2"],
    );
}

#[test]
fn test_invalid_user_reg_ref() {
    expect_compile_errors(
        HardwareSpec {
            num_registers: 1,
            num_stacks: 1,
            max_stack_length: 5,
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
            "Invalid reference to register @ 2:14 to 2:17",
            "Invalid reference to register @ 3:15 to 3:18",
            "Invalid reference to register @ 4:13 to 4:16",
            "Invalid reference to register @ 5:13 to 5:16",
            "Invalid reference to register @ 6:13 to 6:16",
            "Invalid reference to register @ 7:13 to 7:16",
            "Invalid reference to register @ 8:14 to 8:17",
            "Invalid reference to register @ 9:16 to 9:19",
        ],
    );
}

#[test]
fn test_invalid_stack_reg_ref() {
    expect_compile_errors(
        HardwareSpec {
            num_registers: 1,
            num_stacks: 1,
            max_stack_length: 5,
        },
        "
        SET RX0 RS1
        ",
        &["Invalid reference to register @ 2:17 to 2:20"],
    );
}

#[test]
fn test_invalid_stack_ref() {
    expect_compile_errors(
        HardwareSpec {
            num_registers: 1,
            num_stacks: 1,
            max_stack_length: 5,
        },
        "
        PUSH 5 S1
        POP S2 RX0
        ",
        &[
            "Invalid reference to stack @ 2:16 to 2:18",
            "Invalid reference to stack @ 3:13 to 3:15",
        ],
    );
}

#[test]
fn test_unwritable_reg() {
    expect_compile_errors(
        HardwareSpec {
            num_registers: 1,
            num_stacks: 1,
            max_stack_length: 5,
        },
        "
        SET RLI 5
        SET RS0 5
        ",
        &[
            "Cannot write to read-only register @ 2:13 to 2:16",
            "Cannot write to read-only register @ 3:13 to 3:16",
        ],
    );
}
