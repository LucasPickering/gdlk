//! Integration tests for GDLK that expect compile errors. The programs in
//! these tests should all fail during compilation.

use gdlk::{ast::LangValue, Compiler, HardwareSpec};

/// Compiles the program for the given hardware, expecting compile error(s).
/// Panics if the program compiles successfully, or if the wrong set of
/// errors is returned.
macro_rules! assert_compile_errors {
    ($hw_spec:expr, $src:expr, $expected_errors:expr $(,)?) => {
        // Compile from hardware+src
        let actual_errors: Vec<String> =
            Compiler::compile($src.into(), $hw_spec)
                .unwrap_err()
                .errors()
                .iter()
                .map(|err| err.to_string())
                .collect();
        let strs: Vec<&str> =
            actual_errors.iter().map(String::as_str).collect();
        assert_eq!(strs.as_slice(), $expected_errors);
    };
}

/// Macro to compile a program and expect a particular compiler error.
macro_rules! assert_parse_error {
    ($src:expr, $expected_error:expr $(,)?) => {
        let actual_errors =
            Compiler::compile($src.into(), HardwareSpec::default())
                .unwrap_err();
        assert_eq!(actual_errors.to_string(), $expected_error);
    };
}

#[test]
fn test_parse_errors_registers() {
    assert_parse_error!(
        "
        READ RX0
        READ RW0
        READ RX0
        ",
        "Syntax error at 3:14: Expected register reference",
    );
    assert_parse_error!(
        "READ",
        "Syntax error at 1:5: Expected register reference",
    );
    assert_parse_error!("WRITE", "Syntax error at 1:6: Expected value");
    assert_parse_error!(
        "READ RW0",
        "Syntax error at 1:6: Expected register reference"
    );
    assert_parse_error!(
        "READ RX01",
        "Syntax error at 1:6: Expected register reference"
    );
}

#[test]
fn test_parse_errors_stacks() {
    assert_parse_error!(
        "PUSH RX0 T0",
        "Syntax error at 1:10: Expected stack reference",
    );
    assert_parse_error!(
        "PUSH RX0 S01",
        "Syntax error at 1:10: Expected stack reference",
    );
}

#[test]
fn test_parse_errors_simple_instructions() {
    assert_parse_error!("RAD RX0", "Syntax error at 1:1: Expected statement");
    assert_parse_error!("READE RX0", "Syntax error at 1:1: Expected statement");
    assert_parse_error!("PUSH STEVE S0", "Syntax error at 1:6: Expected value");
    assert_parse_error!(
        "READ RX1 WRITE RX2",
        "Syntax error at 1:10: Expected end of statement",
    );
}

#[test]
fn test_parse_errors_jumps() {
    // Jumps/labels
    assert_parse_error!("JMP", "Syntax error at 1:4: Expected label");
    assert_parse_error!("JEZ", "Syntax error at 1:4: Expected value");
    assert_parse_error!("JEZ RX0", "Syntax error at 1:8: Expected label");
    assert_parse_error!("JEZ RW0 LABEL", "Syntax error at 1:5: Expected value");
    assert_parse_error!(
        "LABEL:JMP LABEL",
        "Syntax error at 1:7: Expected end of statement"
    );

    // These errors aren't the best, but eh close enough
    assert_parse_error!("JMP BAD-LABEL", "Syntax error at 1:4: Expected label");
    assert_parse_error!(
        "BAD-LABEL:",
        "Syntax error at 1:1: Expected statement"
    );
}

#[test]
fn test_parse_errors_constants() {
    // Float constants
    assert_parse_error!("SET RX0 10.5", "Syntax error at 1:8: Expected value");

    // Out-of-range constants
    assert_parse_error!(
        &format!("SET RX0 {}", LangValue::MAX as i64 + 1),
        "Syntax error at 1:9: Expected value"
    );
    assert_parse_error!(
        &format!("SET RX0 {}", LangValue::MIN as i64 - 1),
        "Syntax error at 1:9: Expected value"
    );
}

#[test]
fn test_parse_empty_file() {
    assert_compile_errors!(
        HardwareSpec::default(),
        "",
        &["Syntax error at 1:1: Expected program"]
    );
    assert_compile_errors!(
        HardwareSpec::default(),
        "    \n\n\t",
        &["Syntax error at 1:1: Expected program"]
    );
}

#[test]
fn test_invalid_user_reg_ref() {
    assert_compile_errors!(
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
            "Validation error at 2:14: Invalid reference to register `RX1`",
            "Validation error at 3:15: Invalid reference to register `RX2`",
            "Validation error at 4:13: Invalid reference to register `RX3`",
            "Validation error at 5:13: Invalid reference to register `RX4`",
            "Validation error at 6:13: Invalid reference to register `RX5`",
            "Validation error at 7:13: Invalid reference to register `RX6`",
            "Validation error at 8:14: Invalid reference to register `RX7`",
            "Validation error at 9:16: Invalid reference to register `RX8`",
        ],
    );
}

#[test]
fn test_invalid_stack_reg_ref() {
    assert_compile_errors!(
        HardwareSpec {
            num_registers: 1,
            num_stacks: 1,
            max_stack_length: 5,
        },
        "
        SET RX0 RS1
        ",
        &["Validation error at 2:17: Invalid reference to register `RS1`"],
    );
}

#[test]
fn test_invalid_stack_ref() {
    assert_compile_errors!(
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
            "Validation error at 2:16: Invalid reference to stack `S1`",
            "Validation error at 3:13: Invalid reference to stack `S2`",
        ],
    );
}

#[test]
fn test_unwritable_reg() {
    assert_compile_errors!(
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
            "Validation error at 2:13: \
                Cannot write to read-only register `RLI`",
            "Validation error at 3:13: \
                Cannot write to read-only register `RS0`",
        ],
    );
}
