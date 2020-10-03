#![deny(clippy::all)]

use crate::utils::{factories::*, ContextBuilder, QueryRunner};
use diesel_factories::Factory;
use gdlk_api::models;
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;

mod utils;

static QUERY: &str = r#"
    mutation ExecuteUserProgramMutation($id: ID!) {
        executeUserProgram(input: { id: $id }) {
            status {
                __typename
            }
        }
    }
"#;

/// Test that the correct values are recorded for each stat type.
#[test]
fn test_execute_user_program_stats() {
    let mut context_builder = ContextBuilder::new();
    let user = context_builder.log_in(&[]);
    let runner = QueryRunner::new(context_builder);
    let conn = runner.db_conn();

    let hw_spec = HardwareSpecFactory::default()
        .name("HW 1")
        .num_registers(2)
        .num_stacks(1)
        .insert(conn);
    let program_spec = ProgramSpecFactory::default()
        .name("prog1")
        .hardware_spec(&hw_spec)
        .input(vec![1])
        .expected_output(vec![1])
        .insert(conn);
    let user_program = UserProgramFactory::default()
        .user(&user)
        .program_spec(&program_spec)
        .file_name("solution.gdlk")
        .source_code(
            "
            READ RX0
            JMP END

            ; these instructions count even though they never get executed
            ; the stack/reg references count too
            PUSH RX0 S0
            POP S0 RX1
            END: ; doesn't count as an instruction
            WRITE RX0
            ",
        )
        .insert(conn);

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(user_program.id.to_string()),
            }
        ),
        (
            json!({
                "executeUserProgram": {
                    "status": {
                        "__typename": "ProgramSuccessOutput",
                        // TODO check stats here once API is fleshed out
                    }
                }
            }),
            vec![]
        )
    );

    // For now, check the PBs because we have a func for it. Once the API is
    // done, we can get the stats directly from the response above
    assert_eq!(
        models::UserProgramRecord::load_pbs_x(conn, user.id, program_spec.id)
            .unwrap(),
        Some(models::UserProgramRecordStats {
            cpu_cycles: 3,
            instructions: 5,
            registers_used: 2,
            stacks_used: 1,
        })
    );
}

/// Test that PB values get updated properly on subsequent executions. We want
/// to make sure that ONLY stats that are actually better get updated. Stats
/// that equal or are worse than the PB shouldn't be written to the PB table.
#[test]
fn test_execute_user_program_pb_update() {
    let mut context_builder = ContextBuilder::new();
    let user = context_builder.log_in(&[]);
    let runner = QueryRunner::new(context_builder);
    let conn = runner.db_conn();

    let hw_spec = HardwareSpecFactory::default()
        .name("HW 1")
        .num_stacks(1)
        .insert(conn);
    let program_spec = ProgramSpecFactory::default()
        .name("prog1")
        .hardware_spec(&hw_spec)
        .input(vec![1])
        .expected_output(vec![1])
        .insert(conn);
    let user_program = UserProgramFactory::default()
        .user(&user)
        .program_spec(&program_spec)
        .file_name("solution.gdlk")
        .source_code("READ RX0\nWRITE RX0")
        .insert(conn);

    // this solution is better in one stat (registers) but worse in another
    // (cpu cycles, instructions)
    let better_user_program = UserProgramFactory::default()
        .user(&user)
        .program_spec(&program_spec)
        .file_name("better.gdlk")
        .source_code("READ RZR\nADD RZR 1\nWRITE 1")
        .insert(conn);

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(user_program.id.to_string()),
            }
        ),
        (
            json!({
                "executeUserProgram": {
                    "status": {
                        "__typename": "ProgramSuccessOutput",
                    }
                }
            }),
            vec![]
        )
    );

    // Check that PBs are correct
    assert_eq!(
        models::UserProgramRecord::load_pbs_x(conn, user.id, program_spec.id)
            .unwrap(),
        Some(models::UserProgramRecordStats {
            cpu_cycles: 2,
            instructions: 2,
            registers_used: 1,
            stacks_used: 0,
        })
    );

    // Test that only PBs that are better get updated
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(better_user_program.id.to_string()),
            }
        ),
        (
            json!({
                "executeUserProgram": {
                    "status": {
                        "__typename": "ProgramSuccessOutput",
                    }
                }
            }),
            vec![]
        )
    );

    // registers_used PB should've improved, but the rest are the same
    assert_eq!(
        models::UserProgramRecord::load_pbs_x(conn, user.id, program_spec.id)
            .unwrap(),
        Some(models::UserProgramRecordStats {
            cpu_cycles: 2,
            instructions: 2,
            registers_used: 0,
            stacks_used: 0,
        })
    );
}

/// Test that we get the correct output format when the program fails to compile
#[test]
fn test_execute_user_program_compile_error() {
    let mut context_builder = ContextBuilder::new();
    let user = context_builder.log_in(&[]);
    let runner = QueryRunner::new(context_builder);
    let conn = runner.db_conn();

    let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(conn);
    let program_spec = ProgramSpecFactory::default()
        .name("prog1")
        .hardware_spec(&hw_spec)
        .insert(conn);
    let user_program = UserProgramFactory::default()
        .user(&user)
        .program_spec(&program_spec)
        .file_name("solution.gdlk")
        .source_code("READ RX0\nWRITE RX2")
        .insert(conn);

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(user_program.id.to_string()),
            }
        ),
        (
            json!({
                "executeUserProgram": {
                    "status": {
                        "__typename": "ProgramCompileError"
                    }
                }
            }),
            vec![]
        )
    );
}

/// Test that we get the correct output format when the program has a runtime
/// error
#[test]
fn test_execute_user_program_runtime_error() {
    let mut context_builder = ContextBuilder::new();
    let user = context_builder.log_in(&[]);
    let runner = QueryRunner::new(context_builder);
    let conn = runner.db_conn();

    let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(conn);
    let program_spec = ProgramSpecFactory::default()
        .name("prog1")
        .hardware_spec(&hw_spec)
        .input(vec![1])
        .expected_output(vec![1])
        .insert(conn);
    let user_program = UserProgramFactory::default()
        .user(&user)
        .program_spec(&program_spec)
        .file_name("solution.gdlk")
        .source_code("READ RX0\nREAD RX0")
        .insert(conn);

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(user_program.id.to_string()),
            }
        ),
        (
            json!({
                "executeUserProgram": {
                    "status": {
                        "__typename": "ProgramRuntimeError"
                    }
                }
            }),
            vec![]
        )
    );
}

/// Test that we get the correct output format when the program terminates
/// but has the incorrect output (doesn't match `expected_output` from the
/// program spec)
#[test]
fn test_execute_user_program_incorrect_output() {
    let mut context_builder = ContextBuilder::new();
    let user = context_builder.log_in(&[]);
    let runner = QueryRunner::new(context_builder);
    let conn = runner.db_conn();

    let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(conn);
    let program_spec = ProgramSpecFactory::default()
        .name("prog1")
        .hardware_spec(&hw_spec)
        .input(vec![1])
        .expected_output(vec![1])
        .insert(conn);
    let user_program = UserProgramFactory::default()
        .user(&user)
        .program_spec(&program_spec)
        .file_name("solution.gdlk")
        .source_code("READ RX0\nWRITE 2")
        .insert(conn);

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(user_program.id.to_string()),
            }
        ),
        (
            json!({
                "executeUserProgram": {
                    "status": {
                        "__typename": "ProgramFailureOutput"
                    }
                }
            }),
            vec![]
        )
    );
}

/// Executing an unknown user_program ID gives a null in the response
#[test]
fn test_execute_user_program_unknown_id() {
    let mut context_builder = ContextBuilder::new();
    context_builder.log_in(&[]);
    let runner = QueryRunner::new(context_builder);

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar("bogus"),
            }
        ),
        (
            json!({
                "executeUserProgram": {
                    "status": serde_json::Value::Null
                }
            }),
            vec![]
        )
    );
}

/// Executing someone else's program should behave like the program doesn't
/// exist
#[test]
fn test_execute_user_program_wrong_owner() {
    let mut context_builder = ContextBuilder::new();
    context_builder.log_in(&[]);
    let runner = QueryRunner::new(context_builder);
    let conn = runner.db_conn();

    let other_user = UserFactory::default().username("other").insert(conn);

    let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(conn);
    let program_spec = ProgramSpecFactory::default()
        .name("prog1")
        .hardware_spec(&hw_spec)
        .insert(conn);
    let user_program = UserProgramFactory::default()
        .user(&other_user)
        .program_spec(&program_spec)
        .file_name("solution.gdlk")
        .source_code("READ RX0\nWRITE RX0")
        .insert(conn);

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(user_program.id.to_string()),
            }
        ),
        (
            json!({
                "executeUserProgram": {
                    "status": serde_json::Value::Null
                }
            }),
            vec![]
        )
    );
}
