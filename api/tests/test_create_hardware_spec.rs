#![deny(clippy::all)]

use diesel::PgConnection;
use gdlk_api::models::{Factory, NewHardwareSpec};
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;
use utils::QueryRunner;

mod utils;

static QUERY: &str = r#"
    mutation CreateHardwareSpecMutation(
        $name: String!,
        $numRegisters: Int!,
        $numStacks: Int!,
        $maxStackLength: Int!,
    ) {
        createHardwareSpec(input: {
            name: $name,
            numRegisters: $numRegisters,
            numStacks: $numStacks,
            maxStackLength: $maxStackLength,
        }) {
            hardwareSpecEdge {
                node {
                    name
                    slug
                    numRegisters
                    numStacks
                    maxStackLength
                }
            }
        }
    }
"#;

/// Test the standard success state of createHardwareSpec
#[test]
fn test_create_hardware_spec_success() {
    let runner = QueryRunner::new();
    let conn: &PgConnection = &runner.db_conn();

    // We'll test collisions against this
    NewHardwareSpec {
        name: "HW 1",
        ..Default::default()
    }
    .create(conn);

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "name" => InputValue::scalar("HW 2"),
                "numRegisters" => InputValue::scalar(3),
                "numStacks" => InputValue::scalar(2),
                "maxStackLength" => InputValue::scalar(16),
            }
        ),
        (
            json!({
                "createHardwareSpec": {
                    "hardwareSpecEdge": {
                        "node": {
                            "name": "HW 2",
                            "slug": "hw-2",
                            "numRegisters": 3,
                            "numStacks": 2,
                            "maxStackLength": 16,
                        }
                    }
                }
            }),
            vec![]
        )
    );
}

/// [ERROR] Test createHardwareSpec when you try to use a pre-existing name
#[test]
fn test_create_hardware_spec_duplicate() {
    let runner = QueryRunner::new();
    let conn: &PgConnection = &runner.db_conn();

    // We'll test collisions against this
    NewHardwareSpec {
        name: "HW 1",
        ..Default::default()
    }
    .create(conn);

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "name" => InputValue::scalar("HW 1"),
                "numRegisters" => InputValue::scalar(3),
                "numStacks" => InputValue::scalar(2),
                "maxStackLength" => InputValue::scalar(16),
            }
        ),
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 8, "column": 9}],
                "message": "This resource already exists",
                "path": ["createHardwareSpec"],
            })]
        )
    );
}

/// [ERROR] Test createHardwareSpec when you pass invalid data
#[test]
fn test_create_hardware_spec_invalid_values() {
    let runner = QueryRunner::new();

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "name" => InputValue::scalar(""),
                "numRegisters" => InputValue::scalar(0),
                "numStacks" => InputValue::scalar(-1),
                "maxStackLength" => InputValue::scalar(-1),
            }
        ),
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 8, "column": 9}],
                "message": "Input validation error(s)",
                "path": ["createHardwareSpec"],
                "extensions": {
                    "name": [{"min": "1", "value": "\"\""}],
                    "num_registers": [{"min": "1.0", "max": "16.0", "value": "0"}],
                    "num_stacks": [{"min": "0.0", "max": "16.0", "value": "-1"}],
                    "max_stack_length": [{"min": "0.0", "max": "256.0", "value": "-1"}],
                }
            })]
        )
    );
}
