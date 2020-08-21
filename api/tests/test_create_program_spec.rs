#![deny(clippy::all)]

use crate::utils::{factories::*, ContextBuilder, QueryRunner};
use diesel_factories::Factory;
use gdlk_api::models::RoleType;
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;

mod utils;

static QUERY: &str = r#"
    mutation CreateProgramSpecMutation(
        $hardwareSpecId: ID!,
        $name: String!,
        $description: String!,
        $input: [Int!]!,
        $expectedOutput: [Int!]!,
    ) {
        createProgramSpec(input: {
            hardwareSpecId: $hardwareSpecId,
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

/// Create a program spec successfully
#[test]
fn test_create_program_spec_success() {
    let mut context_builder = ContextBuilder::new();
    context_builder.log_in(&[RoleType::SpecCreator]);
    let conn = context_builder.db_conn();

    let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(conn);

    let values_list: InputValue = InputValue::list(
        [1, 2, 3].iter().map(|v| InputValue::scalar(*v)).collect(),
    );

    let runner = QueryRunner::new(context_builder);
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "hardwareSpecId" => InputValue::scalar(hw_spec.id.to_string()),
                "name" => InputValue::scalar("Program 2"),
                "description" => InputValue::scalar("description!"),
                "input" => values_list.clone(),
                "expectedOutput" => values_list,
            }
        ),
        (
            json!({
                "createProgramSpec": {
                    "programSpecEdge": {
                        "node": {
                            "name": "Program 2",
                            "slug": "program-2",
                            "description": "description!",
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

/// [ERROR] References an invalid hardware spec
#[test]
fn test_create_program_spec_invalid_hw_spec() {
    let mut context_builder = ContextBuilder::new();
    context_builder.log_in(&[RoleType::SpecCreator]);
    let runner = QueryRunner::new(context_builder);

    let values_list: InputValue = InputValue::list(
        [1, 2, 3].iter().map(|v| InputValue::scalar(*v)).collect(),
    );

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "hardwareSpecId" => InputValue::scalar("bad"),
                "name" => InputValue::scalar("Program 3"),
                "description" => InputValue::scalar("description!"),
                "input" => values_list.clone(),
                "expectedOutput" => values_list,
            }
        ),
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 9, "column": 9}],
                "message": "Not found",
                "path": ["createProgramSpec"],
            })]
        )
    );
}

/// [ERROR] Program spec name is already taken
#[test]
fn test_create_program_spec_duplicate() {
    let mut context_builder = ContextBuilder::new();
    context_builder.log_in(&[RoleType::SpecCreator]);
    let conn = context_builder.db_conn();

    let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(conn);
    // We'll test collisions against this
    ProgramSpecFactory::default()
        .name("program 1")
        .hardware_spec(&hw_spec)
        .insert(conn);

    let values_list: InputValue = InputValue::list(
        [1, 2, 3].iter().map(|v| InputValue::scalar(*v)).collect(),
    );

    let runner = QueryRunner::new(context_builder);
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "hardwareSpecId" => InputValue::scalar(hw_spec.id.to_string()),
                "name" => InputValue::scalar("Program 1"),
                "description" => InputValue::scalar("description!"),
                "input" => values_list.clone(),
                "expectedOutput" => values_list,
            }
        ),
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 9, "column": 9}],
                "message": "This resource already exists",
                "path": ["createProgramSpec"],
            })]
        )
    );
}

/// [ERROR] Values given are invalid
#[test]
fn test_create_program_spec_invalid_values() {
    let mut context_builder = ContextBuilder::new();
    context_builder.log_in(&[RoleType::SpecCreator]);
    let conn = context_builder.db_conn();

    let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(conn);
    let values_list: InputValue = InputValue::list(
        [1, 2, 3].iter().map(|v| InputValue::scalar(*v)).collect(),
    );

    let runner = QueryRunner::new(context_builder);
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "hardwareSpecId" => InputValue::scalar(hw_spec.id.to_string()),
                "name" => InputValue::scalar(""),
                "description" => InputValue::scalar("description!"),
                // TODO use invalid values here once the DB validation is working
                "input" => values_list.clone(),
                "expectedOutput" => values_list,
            }
        ),
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 9, "column": 9}],
                "message": "Input validation error(s)",
                "path": ["createProgramSpec"],
                "extensions": {
                    "name": [{"min": "1", "value": "\"\""}],
                }
            })]
        )
    );
}

/// [ERROR] You have to be logged in to do this
#[test]
fn test_create_program_spec_not_logged_in() {
    let context_builder = ContextBuilder::new();
    let conn = context_builder.db_conn();

    let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(conn);
    let values_list: InputValue = InputValue::list(
        [1, 2, 3].iter().map(|v| InputValue::scalar(*v)).collect(),
    );

    let runner = QueryRunner::new(context_builder);
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "hardwareSpecId" => InputValue::scalar(hw_spec.id.to_string()),
                "name" => InputValue::scalar(""),
                "description" => InputValue::scalar("description!"),
                "input" => values_list.clone(),
                "expectedOutput" => values_list,
            }
        ),
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 9, "column": 9}],
                "message": "Not logged in",
                "path": ["createProgramSpec"],
            })]
        )
    );
}

/// [ERROR] You need permission to do this
#[test]
fn test_create_program_spec_permission_denied() {
    let mut context_builder = ContextBuilder::new();
    context_builder.log_in(&[]); // Insufficient permissions
    let conn = context_builder.db_conn();

    let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(conn);
    let values_list: InputValue = InputValue::list(
        [1, 2, 3].iter().map(|v| InputValue::scalar(*v)).collect(),
    );

    let runner = QueryRunner::new(context_builder);
    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "hardwareSpecId" => InputValue::scalar(hw_spec.id.to_string()),
                "name" => InputValue::scalar(""),
                "description" => InputValue::scalar("description!"),
                "input" => values_list.clone(),
                "expectedOutput" => values_list,
            }
        ),
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 9, "column": 9}],
                "message": "Insufficient permissions to perform this action",
                "path": ["createProgramSpec"],
            })]
        )
    );
}
