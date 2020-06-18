#![deny(clippy::all, unused_must_use, unused_imports)]

use diesel::PgConnection;
use gdlk_api::models::{Factory, NewHardwareSpec, NewProgramSpec};
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;
use utils::QueryRunner;

mod utils;

#[test]
fn test_field_hardware_spec() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.db_conn();

    let hardware_spec_id = NewHardwareSpec {
        name: "hw1",
        ..Default::default()
    }
    .create(conn)
    .id;
    NewProgramSpec {
        name: "prog1",
        hardware_spec_id,
        ..Default::default()
    }
    .create(conn);
    let query = r#"
        query HardwareSpecQuery($slug: String!) {
            hardwareSpec(slug: $slug) {
                id
                slug
                numRegisters
                numStacks
                maxStackLength
            }
        }
    "#;

    // Known hardware spec
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "slug" => InputValue::scalar("hw1"),
                "programSpecSlug" => InputValue::scalar("prog1"),
            }
        ),
        (
            json!({
                "hardwareSpec": {
                    "id": (hardware_spec_id.to_string()),
                    "slug": "hw1",
                    "numRegisters": 1,
                    "numStacks": 0,
                    "maxStackLength": 0,
                }
            }),
            vec![]
        )
    );

    // Unknown hardware spec
    assert_eq!(
        runner.query(
            query,
            hashmap! { "slug" => InputValue::scalar("unknown_hw_spec") }
        ),
        (json!({ "hardwareSpec": serde_json::Value::Null }), vec![])
    );
}

#[test]
fn test_field_hardware_specs() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.db_conn();

    NewHardwareSpec {
        name: "hw1",
        ..Default::default()
    }
    .create(conn);
    NewHardwareSpec {
        name: "hw2",
        ..Default::default()
    }
    .create(conn);
    NewHardwareSpec {
        name: "hw3",
        ..Default::default()
    }
    .create(conn);
    let query = r#"
        query HardwareSpecsQuery($first: Int, $after: Cursor) {
            hardwareSpecs(first: $first, after: $after) {
                totalCount
                pageInfo {
                    startCursor
                    endCursor
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    cursor
                    node {
                        slug
                    }
                }
            }
        }
    "#;

    // Query all nodes
    assert_eq!(
        runner.query(query, hashmap! {}),
        (
            json!({
                "hardwareSpecs": {
                    "totalCount": 3,
                    "pageInfo": {
                        "startCursor": "AAAAAA==",
                        "endCursor": "AAAAAg==",
                        "hasPreviousPage": false,
                        "hasNextPage": false,
                    },
                    "edges": [
                        {
                            "cursor": "AAAAAA==",
                            "node": {
                                "slug": "hw1"
                            }
                        },
                        {
                            "cursor": "AAAAAQ==",
                            "node": {
                                "slug": "hw2"
                            }
                        },
                        {
                            "cursor": "AAAAAg==",
                            "node": {
                                "slug": "hw3"
                            }
                        },
                    ]
                }
            }),
            vec![]
        )
    );

    // Query just one node
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "first" => InputValue::scalar(1),
                "after" => InputValue::scalar("AAAAAA=="),
            }
        ),
        (
            json!({
                "hardwareSpecs": {
                    "totalCount": 3,
                    "pageInfo": {
                        "startCursor": "AAAAAQ==",
                        "endCursor": "AAAAAQ==",
                        "hasPreviousPage": true,
                        "hasNextPage": true,
                    },
                    "edges": [
                        {
                            "cursor": "AAAAAQ==",
                            "node": {
                                "slug": "hw2"
                            }
                        }
                    ]
                }
            }),
            vec![]
        )
    );
}

#[test]
fn test_field_hardware_spec_program_spec() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.db_conn();

    let hardware_spec_id = NewHardwareSpec {
        name: "hw1",
        ..Default::default()
    }
    .create(conn)
    .id;
    NewProgramSpec {
        name: "prog1",
        hardware_spec_id,
        ..Default::default()
    }
    .create(conn);
    NewProgramSpec {
        name: "prog2",
        hardware_spec_id,
        ..Default::default()
    }
    .create(conn);
    NewProgramSpec {
        name: "prog3",
        hardware_spec_id,
        ..Default::default()
    }
    .create(conn);

    let hardware_spec2_id = NewHardwareSpec {
        name: "hw2",
        ..Default::default()
    }
    .create(conn)
    .id;
    NewProgramSpec {
        name: "prog1",
        hardware_spec_id: hardware_spec2_id,
        ..Default::default()
    }
    .create(conn);

    let query = r#"
        query HardwareSpecQuery(
            $slug: String!,
            $progSlug: String!,
            $progFirst: Int,
        ) {
            hardwareSpec(slug: $slug) {
                programSpec(slug: $progSlug) {
                    slug
                }
                programSpecs(first: $progFirst) {
                    totalCount
                    edges {
                        node {
                            slug
                        }
                    }
                }
            }
        }
    "#;

    // Known program spec
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "slug" => InputValue::scalar("hw1"),
                "progSlug" => InputValue::scalar("prog1"),
            }
        ),
        (
            json!({
                "hardwareSpec": {
                    "programSpec": {
                        "slug": "prog1",
                    },
                    "programSpecs": {
                        "totalCount": 3,
                        "edges": [
                            {
                                "node": {
                                    "slug": "prog1",
                                }
                            },
                            {
                                "node": {
                                    "slug": "prog2",
                                }
                            },
                            {
                                "node": {
                                    "slug": "prog3",
                                }
                            },
                        ]
                    }
                }
            }),
            vec![]
        )
    );

    // Unknown program spec
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "slug" => InputValue::scalar("hw1"),
                "progSlug" => InputValue::scalar("unknown_prog"),
                "progFirst" => InputValue::scalar(0),
            }
        ),
        (
            json!({
                "hardwareSpec": {
                    "programSpec": serde_json::Value::Null,
                    "programSpecs": {
                        "totalCount": 3,
                        "edges": []
                    }
                }
            }),
            vec![]
        )
    );
}
