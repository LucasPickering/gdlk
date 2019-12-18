//! Integration tests for GDLK that expect program success. The programs in
//! these tests should compile successfully, and execute with a successful
//! outcome.

use gdlk::{
    ast::LangValue, compile, HardwareSpec, Machine, ProgramSpec,
    MAX_CYCLE_COUNT,
};

/// Compiles the program for the given hardware, and executes it against the
/// program spec. Panics if the compile fails or the execution isn't
/// successful.
fn execute_expect_success(
    hardware_spec: HardwareSpec,
    program_spec: ProgramSpec,
    src: &str,
) -> Machine {
    // Compile from hardware+src
    let mut machine =
        compile(&hardware_spec, &program_spec, src.into()).unwrap();

    // Execute to completion
    let success = machine.execute_all().unwrap();

    // Make sure program terminated successfully
    // Check each bit of state individually to make debugging easier
    assert_eq!(machine.input, Vec::new() as Vec<LangValue>);
    assert_eq!(machine.output, program_spec.expected_output);
    // Final sanity check, in case we change the criteria for success
    assert!(success);
    machine
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
fn test_cmp() {
    execute_expect_success(
        HardwareSpec {
            num_registers: 2,
            num_stacks: 0,
            max_stack_length: 0,
        },
        ProgramSpec {
            input: vec![],
            expected_output: vec![-1, 0, 1, 1],
        },
        "
        CMP RX0 1 2
        WRITE RX0
        CMP RX0 2 2
        WRITE RX0
        CMP RX0 10 0
        WRITE RX0

        SET RX0 3
        SET RX1 1
        CMP RX0 RX0 RX1
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

#[test]
fn test_insertion_sort() {
    execute_expect_success(
        HardwareSpec {
            num_registers: 3,
            num_stacks: 2,
            max_stack_length: 16,
        },
        ProgramSpec {
            input: vec![9, 3, 8, 4, 5, 1, 3, 8, 9, 5, 2, 10, 4, 1, 8],
            expected_output: vec![1, 1, 2, 3, 3, 4, 4, 5, 5, 8, 8, 8, 9, 9, 10],
        },
        "
        ; RX0:  the last element pulled off the input
        ; RX1:  the current element in the sorted list we're comparing to
        ; RX2:  scratch space for comparisons and such
        ; S0:   the sorted list so far, with greatest at the bottom
        ; S1:   scratch spaced used during insertion, to hold the chunk of the
        ;       list that's less than RX0
        WHILE RLI {
            READ RX0
            SET RX2 RS0
            WHILE RX2 {
                POP S0 RX1

                ; check if
                CMP RX2 RX0 RX1
                SUB RX2 1
                IF RX2 {
                    SET RX2 -1
                }
                IF RX2 { ; RX0 <= RX1
                    PUSH RX1 S0 ; RX1 > RX0, put it back on the stack
                }
                ADD RX2 1
                IF RX2 { ; RX0 > RX1
                    PUSH RX1 S1
                }
                MUL RX2 RS0 ; iterate if RX0 > RX1 and there's more left in S0
            }
            PUSH RX0 S0
            WHILE RS1 {
                POP S1 RX1
                PUSH RX1 S0
            }
        }

        ; write the sorted list to output
        WHILE RS0 {
            POP S0 RX0
            WRITE RX0
        }
        ",
    );
}

#[test]
fn test_cycle_count_simple() {
    let machine = execute_expect_success(
        HardwareSpec::default(),
        ProgramSpec {
            input: vec![1],
            expected_output: vec![2],
        },
        "
        READ RX0
        ADD RX0 1
        WRITE RX0
        ",
    );
    assert_eq!(machine.cycle_count, 3);
}

#[test]
fn test_cycle_count_if() {
    let m1 = execute_expect_success(
        HardwareSpec::default(),
        ProgramSpec {
            input: vec![],
            expected_output: vec![],
        },
        "
        SET RX0 0
        IF RX0 {}

        SET RX0 1
        IF RX0 {}
        ",
    );
    // IF counts as one instruction, regardless of the value of the condition
    assert_eq!(m1.cycle_count, 4);
}

#[test]
fn test_cycle_count_while() {
    let m1 = execute_expect_success(
        HardwareSpec::default(),
        ProgramSpec {
            input: vec![1, 2, 3],
            expected_output: vec![1, 2, 3],
        },
        "
        WHILE RLI {
            READ RX0
            WRITE RX0
        }
        ",
    );
    // The initial WHILE check counts as one instruction, plus one more for
    // each subsequent loop
    assert_eq!(m1.cycle_count, 10);
}

#[test]
fn test_equal_max_cycle_count() {
    // We can exit successfully with exactly the maximum number of cycles
    let machine = execute_expect_success(
        HardwareSpec::default(),
        ProgramSpec {
            input: vec![(MAX_CYCLE_COUNT as i32 - 1) / 2],
            expected_output: vec![],
        },
        "
        READ RX0
        WHILE RX0 {
            SUB RX0 1
        }
        ",
    );
    assert_eq!(machine.cycle_count, MAX_CYCLE_COUNT);
}
