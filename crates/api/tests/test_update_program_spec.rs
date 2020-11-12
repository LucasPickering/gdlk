#![deny(clippy::all)]

use crate::utils::{factories::*, QueryRunner};
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
#[actix_rt::test]
async fn test_update_program_spec_partial_modification() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[RoleType::Admin]);

    let program_spec = runner.run_with_conn(|conn| {
        let hardware_spec =
            HardwareSpecFactory::default().name("HW 1").insert(&conn);
        // This is the one we'll be modifying
        ProgramSpecFactory::default()
            .name("Program 2")
            .hardware_spec(&hardware_spec)
            .insert(&conn)
    });

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar(program_spec.id.to_string()),
                    "description" => InputValue::scalar("description!")
                }
            )
            .await,
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
#[actix_rt::test]
async fn test_update_program_spec_full_modification() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[RoleType::Admin]);

    let program_spec = runner.run_with_conn(|conn| {
        let hardware_spec =
            HardwareSpecFactory::default().name("HW 1").insert(&conn);
        ProgramSpecFactory::default()
            .name("Program 2")
            .hardware_spec(&hardware_spec)
            .insert(&conn)
    });

    let values_list: InputValue = InputValue::list(
        [1, 2, 3].iter().map(|v| InputValue::scalar(*v)).collect(),
    );

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar(program_spec.id.to_string()),
                    "name" => InputValue::scalar("Program 22"),
                    "description" => InputValue::scalar("new description!"),
                    "input" => values_list.clone(),
                    "expectedOutput" => values_list,
                }
            )
            .await,
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
#[actix_rt::test]
async fn test_update_program_spec_invalid_id() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[RoleType::Admin]);

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar("bad"),
                    "name" => InputValue::scalar("Program 3"),
                }
            )
            .await,
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

#[actix_rt::test]
async fn test_update_program_spec_empty_modification() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[RoleType::Admin]);

    let program_spec = runner.run_with_conn(|conn| {
        let hardware_spec =
            HardwareSpecFactory::default().name("HW 1").insert(&conn);
        // We'll test collisions against this
        ProgramSpecFactory::default()
            .name("Program 1")
            .hardware_spec(&hardware_spec)
            .insert(&conn);
        // This is the one we'll actually be modifying
        ProgramSpecFactory::default()
            .name("Program 2")
            .hardware_spec(&hardware_spec)
            .insert(&conn)
    });

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar(program_spec.id.to_string()),
                }
            )
            .await,
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

#[actix_rt::test]
async fn test_update_program_spec_duplicate() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[RoleType::Admin]);

    let program_spec = runner.run_with_conn(|conn| {
        let hardware_spec =
            HardwareSpecFactory::default().name("HW 1").insert(&conn);
        // We'll test collisions against this
        ProgramSpecFactory::default()
            .name("Program 1")
            .hardware_spec(&hardware_spec)
            .insert(&conn);
        // This is the one we'll actually be modifying
        ProgramSpecFactory::default()
            .name("Program 2")
            .hardware_spec(&hardware_spec)
            .insert(&conn)
    });

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar(program_spec.id.to_string()),
                    "name" => InputValue::scalar("Program 1"),
                }
            )
            .await,
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

#[actix_rt::test]
async fn test_update_program_spec_invalid_values() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[RoleType::Admin]);

    let program_spec = runner.run_with_conn(|conn| {
        let hardware_spec =
            HardwareSpecFactory::default().name("HW 1").insert(&conn);
        ProgramSpecFactory::default()
            .name("Program 2")
            .hardware_spec(&hardware_spec)
            .insert(&conn)
    });

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar(program_spec.id.to_string()),
                    "name" => InputValue::scalar(""),
                }
            )
            .await,
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
#[actix_rt::test]
async fn test_update_program_spec_not_logged_in() {
    let runner = QueryRunner::new();

    let program_spec = runner.run_with_conn(|conn| {
        let hardware_spec =
            HardwareSpecFactory::default().name("HW 1").insert(&conn);
        ProgramSpecFactory::default()
            .name("Program 2")
            .hardware_spec(&hardware_spec)
            .insert(&conn)
    });

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar(program_spec.id.to_string()),
                    "name" => InputValue::scalar(""),
                }
            )
            .await,
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
#[actix_rt::test]
async fn test_update_program_spec_permission_denied() {
    let mut runner = QueryRunner::new();
    runner.log_in(&[]); // Insufficient permissions

    let program_spec = runner.run_with_conn(|conn| {
        let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(&conn);
        ProgramSpecFactory::default()
            .name("Program 2")
            .hardware_spec(&hw_spec)
            .insert(&conn)
    });

    assert_eq!(
        runner
            .query(
                QUERY,
                hashmap! {
                    "id" => InputValue::scalar(program_spec.id.to_string()),
                    "name" => InputValue::scalar(""),
                }
            )
            .await,
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
