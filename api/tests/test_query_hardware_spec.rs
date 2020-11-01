#![deny(clippy::all)]

use crate::utils::{factories::*, QueryRunner};
use diesel_factories::Factory;
use juniper::InputValue;
use maplit::hashmap;
use serde_json::json;

mod utils;

#[actix_rt::test]
async fn test_field_hardware_spec() {
    let runner = QueryRunner::new();

    let hardware_spec = runner.run_with_conn(|conn| {
        let hardware_spec =
            HardwareSpecFactory::default().name("hw1").insert(&conn);
        ProgramSpecFactory::default()
            .name("prog1")
            .hardware_spec(&hardware_spec)
            .insert(&conn);
        hardware_spec
    });

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
        runner
            .query(
                query,
                hashmap! {
                    "slug" => InputValue::scalar("hw1"),
                    "programSpecSlug" => InputValue::scalar("prog1"),
                }
            )
            .await,
        (
            json!({
                "hardwareSpec": {
                    "id": (hardware_spec.id.to_string()),
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
        runner
            .query(
                query,
                hashmap! { "slug" => InputValue::scalar("unknown_hw_spec") }
            )
            .await,
        (json!({ "hardwareSpec": serde_json::Value::Null }), vec![])
    );
}

#[actix_rt::test]
async fn test_field_hardware_specs() {
    let runner = QueryRunner::new();

    runner.run_with_conn(|conn| {
        HardwareSpecFactory::default().name("hw1").insert(&conn);
        HardwareSpecFactory::default().name("hw2").insert(&conn);
        HardwareSpecFactory::default().name("hw3").insert(&conn);
    });

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
        runner.query(query, hashmap! {}).await,
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
        runner
            .query(
                query,
                hashmap! {
                    "first" => InputValue::scalar(1),
                    "after" => InputValue::scalar("AAAAAA=="),
                }
            )
            .await,
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

#[actix_rt::test]
async fn test_field_hardware_spec_program_spec() {
    let runner = QueryRunner::new();

    runner.run_with_conn(|conn| {
        // let hardware_spec_fac=HardwareSpecFactory::default().name("hw1")
        let hardware_spec =
            HardwareSpecFactory::default().name("hw1").insert(&conn);
        ProgramSpecFactory::default()
            .name("prog1")
            .hardware_spec(&hardware_spec)
            .insert(&conn);
        ProgramSpecFactory::default()
            .name("prog2")
            .hardware_spec(&hardware_spec)
            .insert(&conn);
        ProgramSpecFactory::default()
            .name("prog3")
            .hardware_spec(&hardware_spec)
            .insert(&conn);

        let hardware_spec2 =
            HardwareSpecFactory::default().name("hw2").insert(&conn);
        ProgramSpecFactory::default()
            .name("prog1")
            .hardware_spec(&hardware_spec2)
            .insert(&conn)
    });

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
        runner
            .query(
                query,
                hashmap! {
                    "slug" => InputValue::scalar("hw1"),
                    "progSlug" => InputValue::scalar("prog1"),
                }
            )
            .await,
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
        runner
            .query(
                query,
                hashmap! {
                    "slug" => InputValue::scalar("hw1"),
                    "progSlug" => InputValue::scalar("unknown_prog"),
                    "progFirst" => InputValue::scalar(0),
                }
            )
            .await,
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
