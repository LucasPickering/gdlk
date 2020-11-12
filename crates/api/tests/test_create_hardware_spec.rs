#![deny(clippy::all)]

use crate::utils::{factories::HardwareSpecFactory, QueryRunner};
use diesel_factories::Factory;
use gdlk_api::models::RoleType;
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;

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
#[actix_rt::test]
async fn test_create_hardware_spec_success() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[RoleType::SpecCreator]);

    // We'll test collisions against this
    runner.run_with_conn(|conn| {
        HardwareSpecFactory::default().name("HW 1").insert(&conn);
    });

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "name" => InputValue::scalar("HW 2"),
                    "numRegisters" => InputValue::scalar(3),
                    "numStacks" => InputValue::scalar(2),
                    "maxStackLength" => InputValue::scalar(16),
                }
            )
            .await,
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
#[actix_rt::test]
async fn test_create_hardware_spec_duplicate() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[RoleType::SpecCreator]);

    runner.run_with_conn(|conn| {
        // We'll test collisions against this
        HardwareSpecFactory::default().name("HW 1").insert(&conn);
    });

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "name" => InputValue::scalar("HW 1"),
                    "numRegisters" => InputValue::scalar(3),
                    "numStacks" => InputValue::scalar(2),
                    "maxStackLength" => InputValue::scalar(16),
                }
            )
            .await,
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
#[actix_rt::test]
async fn test_create_hardware_spec_invalid_values() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[RoleType::SpecCreator]);

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "name" => InputValue::scalar(""),
                    "numRegisters" => InputValue::scalar(0),
                    "numStacks" => InputValue::scalar(-1),
                    "maxStackLength" => InputValue::scalar(-1),
                }
            )
            .await,
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

/// [ERROR] Test createHardwareSpec when you aren't logged in
#[actix_rt::test]
async fn test_create_hardware_spec_not_logged_in() {
    let runner = QueryRunner::new();

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "name" => InputValue::scalar(""),
                    "numRegisters" => InputValue::scalar(0),
                    "numStacks" => InputValue::scalar(-1),
                    "maxStackLength" => InputValue::scalar(-1),
                }
            )
            .await,
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 8, "column": 9}],
                "message": "Not logged in",
                "path": ["createHardwareSpec"],
            })]
        )
    );
}

/// [ERROR] Test createHardwareSpec when you don't have the proper permission
#[actix_rt::test]
async fn test_create_hardware_spec_permission_denied() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[]); // Insufficient permissions

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "name" => InputValue::scalar(""),
                    "numRegisters" => InputValue::scalar(0),
                    "numStacks" => InputValue::scalar(-1),
                    "maxStackLength" => InputValue::scalar(-1),
                }
            )
            .await,
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 8, "column": 9}],
                "message": "Insufficient permissions to perform this action",
                "path": ["createHardwareSpec"],
            })]
        )
    );
}
