//! Integration tests for the GDLK Wasm API

use gdlk_wasm::{
    compile, HardwareSpec, LangValue, ProgramSpec, SourceElement, Span,
};
use maplit::hashmap;
use std::collections::HashMap;
use wasm_bindgen_test::wasm_bindgen_test;

/// Checks each portion of the given machine's state, and compares each field
/// to the corresponding expected value.
macro_rules! assert_machine_state {
    (
        $machine:expr,
        program_counter = $program_counter:expr,
        cycle_count = $cycle_count:expr,
        terminated = $terminated:expr,
        successful = $successful:expr,
        input = $input:expr,
        output = $output:expr,
        registers = $registers:expr,
        stacks = $stacks:expr,
        error = $error:expr
        $(,)?
    ) => {{
        let m = &$machine;
        assert_eq!(m.program_counter(), $program_counter, "program counter");
        assert_eq!(m.cycle_count(), $cycle_count, "cycle count");
        assert_eq!(m.terminated(), $terminated, "terminated");
        assert_eq!(m.successful(), $successful, "successful");
        assert_eq!(m.input(), $input as &[LangValue], "input");
        assert_eq!(m.output(), $output as &[LangValue], "output");
        assert_eq!(
            m.wasm_registers()
                .into_serde::<HashMap<String, LangValue>>()
                .unwrap(),
            $registers,
            "registers"
        );
        assert_eq!(
            m.wasm_stacks()
                .into_serde::<HashMap<String, Vec<LangValue>>>()
                .unwrap(),
            $stacks,
            "stacks"
        );
        assert_eq!(m.wasm_error(), $error, "error");
    }};
}

#[wasm_bindgen_test]
fn test_compile_success() {
    let result = compile(
        &HardwareSpec {
            num_registers: 1,
            num_stacks: 2,
            max_stack_length: 10,
        },
        &ProgramSpec::new(vec![1], vec![1]),
        "
        READ RX0
        WRITE RX0
        ",
    );

    let compile_success = result.unwrap();
    let instructions = compile_success.instructions();
    assert_eq!(
        instructions.into_serde::<Vec<SourceElement>>().unwrap(),
        vec![
            SourceElement {
                text: "TODO".into(),
                span: Span {
                    offset: 9,
                    length: 8,
                    start_line: 2,
                    start_col: 9,
                    end_line: 2,
                    end_col: 17
                }
            },
            SourceElement {
                text: "TODO".into(),
                span: Span {
                    offset: 26,
                    length: 9,
                    start_line: 3,
                    start_col: 9,
                    end_line: 3,
                    end_col: 18
                }
            }
        ]
    );

    let machine = compile_success.machine();

    // Test initial state
    assert_machine_state!(
        machine,
        program_counter = 0,
        cycle_count = 0,
        terminated = false,
        successful = false,
        input = &[1i16],
        output = &[],
        registers = hashmap! {
            "RLI".into() => 1,
            "RS0".into() => 0,
            "RS1".into() => 0,
            "RX0".into() => 0,
        },
        stacks = hashmap! {
            "S0".into() => vec![],
            "S1".into() => vec![],
        },
        error = None
    );
}

#[wasm_bindgen_test]
fn test_compile_errors() {
    let result = compile(
        &HardwareSpec::default(),
        &ProgramSpec::default(),
        "
        READ RX1
        PUSH 3 S0
        ",
    );

    let errors = result.unwrap_err();
    assert_eq!(
        errors.into_serde::<Vec<SourceElement>>().unwrap(),
        vec![
            SourceElement {
                text:
                    "Validation error at 2:14: Invalid reference to register `RX1`"
                        .into(),
                span: Span {
                    offset: 14,
                    length: 3,
                    start_line: 2,
                    start_col: 14,
                    end_line: 2,
                    end_col: 17,
                }
            },
            SourceElement {
                text: "Validation error at 3:16: Invalid reference to stack `S0`"
                    .into(),
                span: Span {
                    offset: 33,
                    length: 2,
                    start_line: 3,
                    start_col: 16,
                    end_line: 3,
                    end_col: 18,
                }
            }
        ]
    );
}

#[wasm_bindgen_test]
fn test_execute() {
    let result = compile(
        &HardwareSpec {
            num_registers: 1,
            num_stacks: 1,
            max_stack_length: 10,
        },
        &ProgramSpec::new(vec![1, 2, 3], vec![1, 2, 3]),
        "
        START:
            JEZ RLI END
            READ RX0
            PUSH RX0 S0
            POP S0 RX0
            WRITE RX0
            JMP START
        END:
        ",
    );

    let mut machine = result.unwrap().machine();

    // Test initial state
    assert_machine_state!(
        machine,
        program_counter = 0,
        cycle_count = 0,
        terminated = false,
        successful = false,
        input = &[1i16, 2i16, 3i16],
        output = &[],
        registers = hashmap! {
            "RLI".into() => 3,
            "RS0".into() => 0,
            "RX0".into() => 0,
        },
        stacks = hashmap! {
            "S0".into() => vec![],
        },
        error = None
    );

    // JEZ
    assert!(machine.wasm_execute_next());
    assert_machine_state!(
        machine,
        program_counter = 1,
        cycle_count = 1,
        terminated = false,
        successful = false,
        input = &[1i16, 2i16, 3i16],
        output = &[],
        registers = hashmap! {
            "RLI".into() => 3,
            "RS0".into() => 0,
            "RX0".into() => 0,
        },
        stacks = hashmap! {
            "S0".into() => vec![],
        },
        error = None
    );

    // READ
    assert!(machine.wasm_execute_next());
    assert_machine_state!(
        machine,
        program_counter = 2,
        cycle_count = 2,
        terminated = false,
        successful = false,
        input = &[2i16, 3i16],
        output = &[],
        registers = hashmap! {
            "RLI".into() => 2,
            "RS0".into() => 0,
            "RX0".into() => 1,
        },
        stacks = hashmap! {
            "S0".into() => vec![],
        },
        error = None
    );

    // PUSH
    assert!(machine.wasm_execute_next());
    assert_machine_state!(
        machine,
        program_counter = 3,
        cycle_count = 3,
        terminated = false,
        successful = false,
        input = &[2i16, 3i16],
        output = &[],
        registers = hashmap! {
            "RLI".into() => 2,
            "RS0".into() => 1,
            "RX0".into() => 1,
        },
        stacks = hashmap! {
            "S0".into() => vec![1],
        },
        error = None
    );

    // POP
    assert!(machine.wasm_execute_next());
    assert_machine_state!(
        machine,
        program_counter = 4,
        cycle_count = 4,
        terminated = false,
        successful = false,
        input = &[2i16, 3i16],
        output = &[],
        registers = hashmap! {
            "RLI".into() => 2,
            "RS0".into() => 0,
            "RX0".into() => 1,
        },
        stacks = hashmap! {
            "S0".into() => vec![],
        },
        error = None
    );

    // WRITE
    assert!(machine.wasm_execute_next());
    assert_machine_state!(
        machine,
        program_counter = 5,
        cycle_count = 5,
        terminated = false,
        successful = false,
        input = &[2i16, 3i16],
        output = &[1i16],
        registers = hashmap! {
            "RLI".into() => 2,
            "RS0".into() => 0,
            "RX0".into() => 1,
        },
        stacks = hashmap! {
            "S0".into() => vec![],
        },
        error = None
    );

    // JMP
    assert!(machine.wasm_execute_next());
    assert_machine_state!(
        machine,
        program_counter = 0,
        cycle_count = 6,
        terminated = false,
        successful = false,
        input = &[2i16, 3i16],
        output = &[1i16],
        registers = hashmap! {
            "RLI".into() => 2,
            "RS0".into() => 0,
            "RX0".into() => 1,
        },
        stacks = hashmap! {
            "S0".into() => vec![],
        },
        error = None
    );

    // Execute the rest of the program
    while !machine.terminated() {
        assert!(machine.wasm_execute_next());
    }

    // Check final state
    assert_machine_state!(
        machine,
        program_counter = 6,
        cycle_count = 19,
        terminated = true,
        successful = true,
        input = &[],
        output = &[1i16, 2i16, 3i16],
        registers = hashmap! {
            "RLI".into() => 0,
            "RS0".into() => 0,
            "RX0".into() => 3,
        },
        stacks = hashmap! {
            "S0".into() => vec![],
        },
        error = None
    );
}

#[wasm_bindgen_test]
fn test_runtime_error() {
    let result = compile(
        &HardwareSpec::default(),
        &ProgramSpec::default(),
        "READ RX0",
    );

    let mut machine = result.unwrap().machine();
    assert!(machine.wasm_execute_next());
    assert_machine_state!(
        machine,
        program_counter = 0,
        cycle_count = 1,
        terminated = true,
        successful = false,
        input = &[],
        output = &[],
        registers = hashmap! {
            "RLI".into() => 0,
            "RX0".into() => 0,
        },
        stacks = hashmap! {},
        error = Some(SourceElement {
            text: "Runtime error at 1:1: Read attempted while input is empty"
                .into(),
            span: Span {
                offset: 0,
                length: 8,
                start_line: 1,
                start_col: 1,
                end_line: 1,
                end_col: 9
            }
        })
    );
}
