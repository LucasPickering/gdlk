#![deny(clippy::all)]

use crate::utils::{factories::*, QueryRunner};
use diesel_factories::Factory;
use gdlk_api::models::RoleType;
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;

mod utils;

static QUERY: &str = r#"
    mutation UpdateHardwareSpecMutation(
        $id: ID!,
        $name: String,
        $numRegisters: Int,
        $numStacks: Int,
        $maxStackLength: Int,
    ) {
        updateHardwareSpec(input: {
            id: $id,
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

/// Modify just a subset of fields, make sure the others keep their values
#[actix_rt::test]
async fn test_update_hardware_spec_partial_modification() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[RoleType::Admin]);

    let hw_spec = runner.run_with_conn(|conn| {
        HardwareSpecFactory::default().name("HW 2").insert(&conn)
    });

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar(hw_spec.id.to_string()),
                    "numRegisters" => InputValue::scalar(3),
                }
            )
            .await,
        (
            json!({
                "updateHardwareSpec": {
                    "hardwareSpecEdge": {
                        "node": {
                            // these values are all the same as before
                            "slug": "hw-2",
                            "name": "HW 2",
                            "numRegisters": 3,
                            "numStacks": 0,
                            "maxStackLength": 0,
                        }
                    }
                }
            }),
            vec![]
        )
    );
}

/// Modify all fields
#[actix_rt::test]
async fn test_update_hardware_spec_full_modification() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[RoleType::Admin]);

    let hw_spec = runner.run_with_conn(|conn| {
        HardwareSpecFactory::default().name("HW 2").insert(&conn)
    });

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar(hw_spec.id.to_string()),
                    "name" => InputValue::scalar("HW 22"),
                    "numRegisters" => InputValue::scalar(10),
                    "numStacks" => InputValue::scalar(2),
                    "maxStackLength" => InputValue::scalar(16),
                }
            )
            .await,
        (
            json!({
                "updateHardwareSpec": {
                    "hardwareSpecEdge": {
                        "node": {
                            "slug": "hw-2", // slug can't be changed
                            "name": "HW 22",
                            "numRegisters": 10,
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

/// Pass an invalid ID, get null back
#[actix_rt::test]
async fn test_update_hardware_spec_invalid_id() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[RoleType::Admin]);

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar("bad"),
                    "name" => InputValue::scalar("HW 3"),
                }
            )
            .await,
        (
            json!({
                "updateHardwareSpec": {
                    "hardwareSpecEdge": serde_json::Value::Null
                }
            }),
            vec![]
        )
    );
}

/// [ERROR] Test that passing no modifications is an error
#[actix_rt::test]
async fn test_update_hardware_spec_empty_modification() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[RoleType::Admin]);

    let hw_spec = runner.run_with_conn(|conn| {
        HardwareSpecFactory::default().name("HW 2").insert(&conn)
    });

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar(hw_spec.id.to_string()),
                }
            )
            .await,
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 9, "column": 9}],
                "message": "No fields were given to update",
                "path": ["updateHardwareSpec"],
            })]
        )
    );
}

/// [ERROR] Test that using a duplicate name returns an error
#[actix_rt::test]
async fn test_update_hardware_spec_duplicate() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[RoleType::Admin]);

    let hw_spec = runner.run_with_conn(|conn| {
        // We'll test collisions against this
        HardwareSpecFactory::default().name("HW 1").insert(&conn);
        HardwareSpecFactory::default().name("HW 2").insert(&conn)
    });

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar(hw_spec.id.to_string()),
                    "name" => InputValue::scalar("HW 1"),
                }
            )
            .await,
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 9, "column": 9}],
                "message": "This resource already exists",
                "path": ["updateHardwareSpec"],
            })]
        )
    );
}

/// [ERROR] Test that passing invalid values gives an error
#[actix_rt::test]
async fn test_update_hardware_spec_invalid_values() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[RoleType::Admin]);

    let hw_spec = runner.run_with_conn(|conn| {
        HardwareSpecFactory::default().name("HW 2").insert(&conn)
    });

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar(hw_spec.id.to_string()),
                    "name" => InputValue::scalar(""),
                    "numRegisters" => InputValue::scalar(-1),
                    "numStacks" => InputValue::scalar(-1),
                    "maxStackLength" => InputValue::scalar(-1),
                }
            )
            .await,
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 9, "column": 9}],
                "message": "Input validation error(s)",
                "path": ["updateHardwareSpec"],
                "extensions": {
                    "name": [{"min": "1", "value": "\"\""}],
                    "num_registers": [{"min": "1.0", "max": "16.0", "value": "-1"}],
                    "num_stacks": [{"min": "0.0", "max": "16.0", "value": "-1"}],
                    "max_stack_length": [{"min": "0.0", "max": "256.0", "value": "-1"}],
                }
            })]
        )
    );
}

/// [ERROR] Test updateHardwareSpec when you aren't logged in
#[actix_rt::test]
async fn test_update_hardware_spec_not_logged_in() {
    let runner = QueryRunner::new();

    let hw_spec = runner.run_with_conn(|conn| {
        HardwareSpecFactory::default().name("HW 2").insert(&conn)
    });

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar(hw_spec.id.to_string()),
                    "name" => InputValue::scalar(""),
                    "numRegisters" => InputValue::scalar(-1),
                    "numStacks" => InputValue::scalar(-1),
                    "maxStackLength" => InputValue::scalar(-1),
                }
            )
            .await,
        (
            serde_json::Value::Null,
            vec![json!({
                "message": "Not logged in",
                "locations": [{"line": 9, "column": 9}],
                "path": ["updateHardwareSpec"],
            })]
        )
    );
}

/// [ERROR] Test updateHardwareSpec when you don't have permission
#[actix_rt::test]
async fn test_update_hardware_spec_permission_denied() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[]); // Insufficient permissions

    let hw_spec = runner.run_with_conn(|conn| {
        HardwareSpecFactory::default().name("HW 2").insert(&conn)
    });

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar(hw_spec.id.to_string()),
                    "name" => InputValue::scalar(""),
                    "numRegisters" => InputValue::scalar(-1),
                    "numStacks" => InputValue::scalar(-1),
                    "maxStackLength" => InputValue::scalar(-1),
                }
            )
            .await,
        (
            serde_json::Value::Null,
            vec![json!({
                "message": "Insufficient permissions to perform this action",
                "locations": [{"line": 9, "column": 9}],
                "path": ["updateHardwareSpec"],
            })]
        )
    );
}
