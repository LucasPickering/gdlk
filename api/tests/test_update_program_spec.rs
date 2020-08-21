#![deny(clippy::all)]

use crate::utils::{factories::*, ContextBuilder, QueryRunner};
use diesel_factories::Factory;
use gdlk_api::models::RoleType;
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;

mod utils;

static QUERY: &str = r#"
    mutation UpdateProgramSpecMutation(
        $id: ID!,
        $name: String,
        $description: String,
        $input: [Int!],
        $expectedOutput: [Int!],
    ) {
        updateProgramSpec(input: {
            id: $id,
            name: $name,
            description: $description,
            input: $input,
            expectedOutput: $expectedOutput,
        }) {
            programSpecEdge {
                node {
                    name
                    slug
                    description
                    input
                    expectedOutput
                }
            }
        }
    }
"#;

/// Partial modification, make sure unmodified fields keep their old value
#[test]
fn test_update_program_spec_partial_modification() {
    let mut context_builder = ContextBuilder::new();
    context_builder.log_in(&[RoleType::Admin]);
    let conn = context_builder.db_conn();

    let hardware_spec =
        HardwareSpecFactory::default().name("HW 1").insert(conn);
    // This is the one we'll be modifying
    let program_spec = ProgramSpecFactory::default()
        .name("Program 2")
        .hardware_spec(&hardware_spec)
        .insert(conn);

    let runner = QueryRunner::new(context_builder);
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(program_spec.id.to_string()),
                "description" => InputValue::scalar("description!")
            }
        ),
        (
            json!({
                "updateProgramSpec": {
                    "programSpecEdge": {
                        "node": {
                            // these values are all the same as before
                            "slug": "program-2",
                            "name": "Program 2",
                            "description": "description!",
                            "input": [],
                            "expectedOutput": [],
                        }
                    }
                }
            }),
            vec![]
        )
    );
}

/// Modify all fields
#[test]
fn test_update_program_spec_full_modification() {
    let mut context_builder = ContextBuilder::new();
    context_builder.log_in(&[RoleType::Admin]);
    let conn = context_builder.db_conn();

    let hardware_spec =
        HardwareSpecFactory::default().name("HW 1").insert(conn);
    let program_spec = ProgramSpecFactory::default()
        .name("Program 2")
        .hardware_spec(&hardware_spec)
        .insert(conn);

    let values_list: InputValue = InputValue::list(
        [1, 2, 3].iter().map(|v| InputValue::scalar(*v)).collect(),
    );

    let runner = QueryRunner::new(context_builder);
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(program_spec.id.to_string()),
                "name" => InputValue::scalar("Program 22"),
                "description" => InputValue::scalar("new description!"),
                "input" => values_list.clone(),
                "expectedOutput" => values_list,
            }
        ),
        (
            json!({
                "updateProgramSpec": {
                    "programSpecEdge": {
                        "node": {
                            "name": "Program 22",
                            "slug": "program-2", // slug doesn't change
                            "description": "new description!",
                            "input": [1, 2, 3],
                            "expectedOutput": [1, 2, 3],
                        }
                    }
                }
            }),
            vec![]
        )
    );
}

/// Pass an invalid ID, get null back
#[test]
fn test_update_program_spec_invalid_id() {
    let mut context_builder = ContextBuilder::new();
    context_builder.log_in(&[RoleType::Admin]);
    let runner = QueryRunner::new(context_builder);

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar("bad"),
                "name" => InputValue::scalar("Program 3"),
            }
        ),
        (
            json!({
                "updateProgramSpec": {
                    "programSpecEdge": serde_json::Value::Null
                }
            }),
            vec![]
        )
    );
}

#[test]
fn test_update_program_spec_empty_modification() {
    let mut context_builder = ContextBuilder::new();
    context_builder.log_in(&[RoleType::Admin]);
    let conn = context_builder.db_conn();

    let hardware_spec =
        HardwareSpecFactory::default().name("HW 1").insert(conn);
    // We'll test collisions against this
    ProgramSpecFactory::default()
        .name("Program 1")
        .hardware_spec(&hardware_spec)
        .insert(conn);
    // This is the one we'll actually be modifying
    let program_spec = ProgramSpecFactory::default()
        .name("Program 2")
        .hardware_spec(&hardware_spec)
        .insert(conn);

    let runner = QueryRunner::new(context_builder);
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(program_spec.id.to_string()),
            }
        ),
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 9, "column": 9}],
                "message": "No fields were given to update",
                "path": ["updateProgramSpec"],
            })]
        )
    );
}

#[test]
fn test_update_program_spec_duplicate() {
    let mut context_builder = ContextBuilder::new();
    context_builder.log_in(&[RoleType::Admin]);
    let conn = context_builder.db_conn();

    let hardware_spec =
        HardwareSpecFactory::default().name("HW 1").insert(conn);
    // We'll test collisions against this
    ProgramSpecFactory::default()
        .name("Program 1")
        .hardware_spec(&hardware_spec)
        .insert(conn);
    // This is the one we'll actually be modifying
    let program_spec = ProgramSpecFactory::default()
        .name("Program 2")
        .hardware_spec(&hardware_spec)
        .insert(conn);

    let runner = QueryRunner::new(context_builder);
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(program_spec.id.to_string()),
                "name" => InputValue::scalar("Program 1"),
            }
        ),
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 9, "column": 9}],
                "message": "This resource already exists",
                "path": ["updateProgramSpec"],
            })]
        )
    );
}

#[test]
fn test_update_program_spec_invalid_values() {
    let mut context_builder = ContextBuilder::new();
    context_builder.log_in(&[RoleType::Admin]);
    let conn = context_builder.db_conn();

    let hardware_spec =
        HardwareSpecFactory::default().name("HW 1").insert(conn);
    let program_spec = ProgramSpecFactory::default()
        .name("Program 2")
        .hardware_spec(&hardware_spec)
        .insert(conn);

    let runner = QueryRunner::new(context_builder);
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(program_spec.id.to_string()),
                "name" => InputValue::scalar(""),
            }
        ),
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 9, "column": 9}],
                "message": "Input validation error(s)",
                "path": ["updateProgramSpec"],
                "extensions": {
                    "name": [{"min": "1", "value": "\"\""}],
                }
            })]
        )
    );
}

/// [ERROR] You have to be logged in to do this
#[test]
fn test_update_program_spec_not_logged_in() {
    let context_builder = ContextBuilder::new();
    let conn = context_builder.db_conn();

    let hardware_spec =
        HardwareSpecFactory::default().name("HW 1").insert(conn);
    let program_spec = ProgramSpecFactory::default()
        .name("Program 2")
        .hardware_spec(&hardware_spec)
        .insert(conn);

    let runner = QueryRunner::new(context_builder);
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(program_spec.id.to_string()),
                "name" => InputValue::scalar(""),
            }
        ),
        (
            serde_json::Value::Null,
            vec![json!({
                "message": "Not logged in",
                "locations": [{"line": 9, "column": 9}],
                "path": ["updateProgramSpec"],
            })]
        )
    );
}

/// [ERROR] You need permission to do this
#[test]
fn test_update_program_spec_permission_denied() {
    let mut context_builder = ContextBuilder::new();
    context_builder.log_in(&[]); // Insufficient permissions
    let conn = context_builder.db_conn();

    let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(conn);
    let program_spec = ProgramSpecFactory::default()
        .name("Program 2")
        .hardware_spec(&hw_spec)
        .insert(conn);

    let runner = QueryRunner::new(context_builder);
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(program_spec.id.to_string()),
                "name" => InputValue::scalar(""),
            }
        ),
        (
            serde_json::Value::Null,
            vec![json!({
                "message": "Insufficient permissions to perform this action",
                "locations": [{"line": 9, "column": 9}],
                "path": ["updateProgramSpec"],
            })]
        )
    );
}
