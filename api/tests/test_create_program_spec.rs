#![deny(clippy::all)]

use diesel::PgConnection;
use gdlk_api::models::{Factory, NewHardwareSpec, NewProgramSpec};
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;
use utils::QueryRunner;

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
    let runner = QueryRunner::new();
    let conn: &PgConnection = &runner.db_conn();
    let hw_spec = NewHardwareSpec {
        name: "HW 1",
        ..Default::default()
    }
    .create(conn);

    let values_list: InputValue = InputValue::list(
        [1, 2, 3].iter().map(|v| InputValue::scalar(*v)).collect(),
    );

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
    let runner = QueryRunner::new();
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
    let runner = QueryRunner::new();
    let conn: &PgConnection = &runner.db_conn();

    let hw_spec = NewHardwareSpec {
        name: "HW 1",
        ..Default::default()
    }
    .create(conn);
    // We'll test collisions against this
    NewProgramSpec {
        name: "program 1",
        hardware_spec_id: hw_spec.id,
        ..Default::default()
    }
    .create(conn);

    let values_list: InputValue = InputValue::list(
        [1, 2, 3].iter().map(|v| InputValue::scalar(*v)).collect(),
    );

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
    let runner = QueryRunner::new();
    let conn: &PgConnection = &runner.db_conn();
    let hw_spec = NewHardwareSpec {
        name: "HW 1",
        ..Default::default()
    }
    .create(conn);
    let values_list: InputValue = InputValue::list(
        [1, 2, 3].iter().map(|v| InputValue::scalar(*v)).collect(),
    );

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
