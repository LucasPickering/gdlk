use diesel::PgConnection;
use gdlk_api::models::{Factory, NewHardwareSpec};
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;
use utils::QueryRunner;

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
#[test]
fn test_update_hardware_spec_partial_modification() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.db_conn();
    let hw_spec = NewHardwareSpec {
        name: "HW 2",
        ..Default::default()
    }
    .create(conn);

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(hw_spec.id.to_string()),
                "numRegisters" => InputValue::scalar(3),
            }
        ),
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
#[test]
fn test_update_hardware_spec_full_modification() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.db_conn();
    let hw_spec = NewHardwareSpec {
        name: "HW 2",
        ..Default::default()
    }
    .create(conn);

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(hw_spec.id.to_string()),
                "name" => InputValue::scalar("HW 22"),
                "numRegisters" => InputValue::scalar(10),
                "numStacks" => InputValue::scalar(2),
                "maxStackLength" => InputValue::scalar(16),
            }
        ),
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
#[test]
fn test_update_hardware_spec_invalid_id() {
    let runner = QueryRunner::new().unwrap();

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar("bad"),
                "name" => InputValue::scalar("HW 3"),
            }
        ),
        ((
            json!({
                "updateHardwareSpec": {
                    "hardwareSpecEdge": serde_json::Value::Null
                }
            }),
            vec![]
        ))
    );
}

/// [ERROR] Test that passing no modifications is an error
#[test]
fn test_update_hardware_spec_empty_modification() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.db_conn();
    let hw_spec = NewHardwareSpec {
        name: "HW 2",
        ..Default::default()
    }
    .create(conn);

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(hw_spec.id.to_string()),
            }
        ),
        ((
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 9, "column": 9}],
                "message": "No fields were given to update",
                "path": ["updateHardwareSpec"],
            })]
        ))
    );
}

/// [ERROR] Test that using a duplicate name returns an error
#[test]
fn test_update_hardware_spec_duplicate() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.db_conn();

    // We'll test collisions against this
    NewHardwareSpec {
        name: "HW 1",
        ..Default::default()
    }
    .create(conn);
    let hw_spec = NewHardwareSpec {
        name: "HW 2",
        ..Default::default()
    }
    .create(conn);

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(hw_spec.id.to_string()),
                "name" => InputValue::scalar("HW 1"),
            }
        ),
        ((
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 9, "column": 9}],
                "message": "This resource already exists",
                "path": ["updateHardwareSpec"],
            })]
        ))
    );
}

/// [ERROR] Test that passing invalid values gives an error
#[test]
fn test_update_hardware_spec_invalid_values() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.db_conn();
    let hw_spec = NewHardwareSpec {
        name: "HW 2",
        ..Default::default()
    }
    .create(conn);

    assert_eq!(
        runner.query(
            QUERY,
            hashmap! {
                "id" => InputValue::scalar(hw_spec.id.to_string()),
                "name" => InputValue::scalar(""),
                "numRegisters" => InputValue::scalar(-1),
                "numStacks" => InputValue::scalar(-1),
                "maxStackLength" => InputValue::scalar(-1),
            }
        ),
        ((
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
        ))
    );
}
