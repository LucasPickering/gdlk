use diesel::{
    associations::HasTable, result::OptionalExtension, PgConnection, QueryDsl,
    RunQueryDsl,
};
use failure::Fallible;
use gdlk_api::{
    models::{self, Factory},
    schema::user_programs,
    server::{create_gql_schema, Context, GqlSchema},
    util,
};
use juniper::{ExecutionError, InputValue, Variables};
use maplit::hashmap;
use serde::Serialize;
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

/// Macro to run a DELETE statement for each given table. Used to clean up all
/// tables after each test.
macro_rules! delete_tables {
    ($conn:expr, $($model:ty),+ $(,)?) => {
        $(
            diesel::delete(<$model>::table())
            .execute($conn as &PgConnection)
            .unwrap();
        )+
    };
}

fn new_user(username: &str) -> models::NewUser {
    models::NewUser { username }
}

fn new_hardware_spec(name: &str) -> models::NewHardwareSpec {
    models::NewHardwareSpec {
        name,
        num_registers: 1,
        num_stacks: 0,
        max_stack_length: 0,
    }
}

fn new_program_spec(
    name: &str,
    hardware_spec_id: Uuid,
) -> models::NewProgramSpec {
    models::NewProgramSpec {
        name,
        description: "Program spec!",
        hardware_spec_id,
        input: Vec::new(),
        expected_output: Vec::new(),
    }
}

/// Convert a serializable value into a JSON value.
fn to_json<T: Serialize>(input: T) -> serde_json::Value {
    let serialized: String = serde_json::to_string(&input).unwrap();
    serde_json::from_str(&serialized).unwrap()
}

/// Helper type for setting up and executing test GraphQL queries
struct QueryRunner {
    schema: GqlSchema,
    context: Context,
}

impl QueryRunner {
    fn new() -> Fallible<Self> {
        let pool = util::init_db_conn_pool()?;

        Ok(Self {
            schema: create_gql_schema(),
            context: Context {
                pool: Arc::new(pool),
            },
        })
    }

    fn query<'a>(
        &'a self,
        query: &'a str,
        vars: HashMap<&str, InputValue>,
    ) -> (serde_json::Value, Vec<serde_json::Value>) {
        // Map &strs to Strings
        let converted_vars = vars
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect::<Variables>();

        let (data, errors): (juniper::Value<_>, Vec<ExecutionError<_>>) =
            juniper::execute(
                query,
                None,
                &self.schema,
                &converted_vars,
                &self.context,
            )
            .unwrap();

        // Map the output data to JSON, for easier comparison
        (to_json(data), errors.into_iter().map(to_json).collect())
    }
}

// Automatically wipe all tables when the test is done
impl Drop for QueryRunner {
    fn drop(&mut self) {
        let conn = self.context.get_db_conn().unwrap();
        delete_tables!(
            &conn,
            models::UserProgram,
            models::ProgramSpec,
            models::HardwareSpec,
            models::User,
            // Any new table needs to be added here!
        );
    }
}

#[test]
fn test_field_node() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.context.get_db_conn().unwrap();

    let user_id = new_user("user1").create(conn).id;
    let hardware_spec_id = new_hardware_spec("hw1").create(conn).id;
    let program_spec_id =
        new_program_spec("prog1", hardware_spec_id).create(conn).id;
    let user_program_id = models::NewUserProgram {
        user_id,
        program_spec_id,
        file_name: "prog.gdlk",
        source_code: "",
    }
    .create(conn)
    .id;

    let query = r#"
        query NodeQuery($nodeId: ID!) {
            node(id: $nodeId) {
                id
            }
        }
    "#;

    // Check a known UUID for each model type
    for id in &[user_id, hardware_spec_id, program_spec_id, user_program_id] {
        assert_eq!(
            runner.query(
                query,
                hashmap! {
                    "nodeId" => InputValue::scalar(id.to_string()),
                }
            ),
            (
                json!({
                    "node": {
                        "id": id.to_string(),
                    }
                }),
                vec![]
            )
        );
    }

    // Check an invalid UUID, and a valid UUID that isn't in the DB
    for id in &["invalid-uuid", "1bb9a3a1-0149-4264-a0a7-ff17ac7b8a61"] {
        assert_eq!(
            runner.query(
                query,
                hashmap! {
                    "nodeId" => InputValue::scalar(*id),
                }
            ),
            (
                json!({
                    "node": serde_json::Value::Null,
                }),
                vec![]
            )
        );
    }
}

#[test]
fn test_field_user() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.context.get_db_conn().unwrap();

    let user_id = new_user("user1").create(conn).id;
    let query = r#"
        query UserQuery($username: String!) {
            user(username: $username) {
                id
                username
            }
        }
    "#;

    // Known user
    assert_eq!(
        runner.query(
            query,
            hashmap! { "username" => InputValue::scalar("user1") }
        ),
        (
            json!({
                "user": {
                    "id": (user_id.to_string()),
                    "username": "user1"
                }
            }),
            vec![]
        )
    );

    // Unknown user
    assert_eq!(
        runner.query(
            query,
            hashmap! { "username" => InputValue::scalar("unknown_user") }
        ),
        (json!({ "user": serde_json::Value::Null }), vec![])
    );
}

#[test]
fn test_field_hardware_spec() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.context.get_db_conn().unwrap();

    let hardware_spec_id = new_hardware_spec("hw1").create(conn).id;
    new_program_spec("prog1", hardware_spec_id).create(conn);
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
    let conn: &PgConnection = &runner.context.get_db_conn().unwrap();

    new_hardware_spec("hw1").create(conn);
    new_hardware_spec("hw2").create(conn);
    new_hardware_spec("hw3").create(conn);
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
    let conn: &PgConnection = &runner.context.get_db_conn().unwrap();

    let hardware_spec_id = new_hardware_spec("hw1").create(conn).id;
    new_program_spec("prog1", hardware_spec_id).create(conn);
    new_program_spec("prog2", hardware_spec_id).create(conn);
    new_program_spec("prog3", hardware_spec_id).create(conn);

    let hardware_spec2_id = new_hardware_spec("hw2").create(conn).id;
    new_program_spec("prog1", hardware_spec2_id).create(conn);

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

#[test]
fn test_program_spec() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.context.get_db_conn().unwrap();

    let hardware_spec_id = new_hardware_spec("hw1").create(conn).id;
    let program_spec_id = models::NewProgramSpec {
        name: "prog1",
        description: "Program spec!",
        hardware_spec_id,
        input: vec![1, 2, 3],
        expected_output: vec![1, 2, 3],
    }
    .create(conn)
    .id;
    let query = r#"
        query ProgramSpecQuery($hwSlug: String!, $progSlug: String!) {
            hardwareSpec(slug: $hwSlug) {
                programSpec(slug: $progSlug) {
                    id
                    slug
                    input
                    expectedOutput
                    hardwareSpec {
                        slug
                    }
                }
            }
        }
    "#;

    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "hwSlug" => InputValue::scalar("hw1"),
                "progSlug" => InputValue::scalar("prog1"),
            }
        ),
        (
            json!({
                "hardwareSpec": {
                    "programSpec": {
                        "id": program_spec_id.to_string(),
                        "slug": "prog1",
                        "input": vec![1, 2, 3],
                        "expectedOutput": vec![1, 2, 3],
                        "hardwareSpec": {
                            "slug": "hw1"
                        }
                    }
                }
            }),
            vec![]
        )
    );
}

#[test]
fn test_program_spec_user_program() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.context.get_db_conn().unwrap();

    let user_id = new_user("user1").create(conn).id;
    let hardware_spec_id = new_hardware_spec("hw1").create(conn).id;
    let program_spec_id =
        new_program_spec("prog1", hardware_spec_id).create(conn).id;
    let user_program_id = models::NewUserProgram {
        user_id,
        program_spec_id,
        file_name: "sl1.gdlk",
        source_code: "READ RX0",
    }
    .create(conn)
    .id;
    models::NewUserProgram {
        user_id,
        program_spec_id,
        file_name: "sl2.gdlk",
        source_code: "READ RX0",
    }
    .create(conn);
    models::NewUserProgram {
        user_id,
        program_spec_id,
        file_name: "sl3.gdlk",
        source_code: "READ RX0",
    }
    .create(conn);
    // Create a new program spec with a new solution for it
    models::NewUserProgram {
        user_id,
        program_spec_id: new_program_spec("prog2", hardware_spec_id)
            .create(conn)
            .id,
        file_name: "sl1.gdlk",
        source_code: "READ RX0",
    }
    .create(conn);
    let query = r#"
        query UserProgramQuery(
            $hwSlug: String!,
            $progSlug: String!,
            $fileName: String!,
        ) {
            hardwareSpec(slug: $hwSlug) {
                programSpec(slug: $progSlug) {
                    userProgram(fileName: $fileName) {
                        id
                        fileName
                        sourceCode
                        user {
                            username
                        }
                        programSpec {
                            slug
                        }
                    }
                    userPrograms {
                        totalCount
                        edges {
                            node {
                                fileName
                            }
                        }
                    }
                }
            }
        }
    "#;

    // Query a set of user programs
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "hwSlug" => InputValue::scalar("hw1"),
                "progSlug" => InputValue::scalar("prog1"),
                "fileName" => InputValue::scalar("sl1.gdlk"),
            }
        ),
        (
            json!({
                "hardwareSpec": {
                    "programSpec": {
                        "userProgram": {
                            "id": (user_program_id.to_string()),
                            "fileName": "sl1.gdlk",
                            "sourceCode": "READ RX0",
                            "user": {
                                "username": "user1"
                            },
                            "programSpec": {
                                "slug": "prog1"
                            },
                        },
                        "userPrograms": {
                            "totalCount": 3,
                            "edges": [
                                {
                                    "node": {
                                        "fileName": "sl1.gdlk"
                                    }
                                },
                                {
                                    "node": {
                                        "fileName": "sl2.gdlk"
                                    }
                                },
                                {
                                    "node": {
                                        "fileName": "sl3.gdlk"
                                    }
                                },
                            ]
                        }
                    }
                }
            }),
            vec![]
        )
    );
}

#[test]
fn test_create_hardware_spec() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.context.get_db_conn().unwrap();

    // We'll test collisions against this
    new_hardware_spec("HW 1").create(conn);
    let query = r#"
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

    // Success - new hardware spec
    assert_eq!(
        runner.query(
            query,
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

    // Error - duplicate name
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "name" => InputValue::scalar("HW 1"),
                "numRegisters" => InputValue::scalar(3),
                "numStacks" => InputValue::scalar(2),
                "maxStackLength" => InputValue::scalar(16),
            }
        ),
        ((
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 8, "column": 13}],
                "message": "This resource already exists",
                "path": ["createHardwareSpec"],
            })]
        ))
    );

    // Error - invalid values
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "name" => InputValue::scalar(""),
                "numRegisters" => InputValue::scalar(0),
                "numStacks" => InputValue::scalar(-1),
                "maxStackLength" => InputValue::scalar(-1),
            }
        ),
        ((
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 8, "column": 13}],
                "message": "Input validation error(s)",
                "path": ["createHardwareSpec"],
                "extensions": {
                    "name": [{"min": "1", "value": "\"\""}],
                    "num_registers": [{"min": "1.0", "max": "16.0", "value": "0"}],
                    "num_stacks": [{"min": "0.0", "max": "16.0", "value": "-1"}],
                    "max_stack_length": [{"min": "0.0", "max": "256.0", "value": "-1"}],
                }
            })]
        ))
    );
}

#[test]
fn test_update_hardware_spec() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.context.get_db_conn().unwrap();

    // We'll test collisions against this
    new_hardware_spec("HW 1").create(conn);
    let hw_spec = new_hardware_spec("HW 2").create(conn);
    let query = r#"
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

    // Success - partial modification, make sure specified fields keep their old
    // value
    assert_eq!(
        runner.query(
            query,
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

    // Success - modify all fields
    assert_eq!(
        runner.query(
            query,
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

    // Success-ish - invalid ID means just return null
    assert_eq!(
        runner.query(
            query,
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

    // Error - modifying no fields
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "id" => InputValue::scalar(hw_spec.id.to_string()),
            }
        ),
        ((
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 9, "column": 13}],
                "message": "No fields were given to update",
                "path": ["updateHardwareSpec"],
            })]
        ))
    );

    // Error - duplicate name
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "id" => InputValue::scalar(hw_spec.id.to_string()),
                "name" => InputValue::scalar("HW 1"),
            }
        ),
        ((
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 9, "column": 13}],
                "message": "This resource already exists",
                "path": ["updateHardwareSpec"],
            })]
        ))
    );

    // Error - invalid values
    assert_eq!(
        runner.query(
            query,
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
                "locations": [{"line": 9, "column": 13}],
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

#[test]
fn test_create_program_spec() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.context.get_db_conn().unwrap();

    let hw_spec = new_hardware_spec("HW 1").create(conn);
    // We'll test collisions against this
    new_program_spec("program 1", hw_spec.id).create(conn);
    let query = r#"
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

    let values_list: InputValue = InputValue::list(
        [1, 2, 3].iter().map(|v| InputValue::scalar(*v)).collect(),
    );

    // Success - new program spec
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "hardwareSpecId" => InputValue::scalar(hw_spec.id.to_string()),
                "name" => InputValue::scalar("Program 2"),
                "description" => InputValue::scalar("description!"),
                "input" => values_list.clone(),
                "expectedOutput" => values_list.clone(),
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

    // Error - invalid hardware spec ID
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "hardwareSpecId" => InputValue::scalar("bad"),
                "name" => InputValue::scalar("Program 3"),
                "description" => InputValue::scalar("description!"),
                "input" => values_list.clone(),
                "expectedOutput" => values_list.clone(),
            }
        ),
        ((
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 9, "column": 13}],
                "message": "Not found",
                "path": ["createProgramSpec"],
            })]
        ))
    );

    // Error - duplicate name
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "hardwareSpecId" => InputValue::scalar(hw_spec.id.to_string()),
                "name" => InputValue::scalar("Program 1"),
                "description" => InputValue::scalar("description!"),
                "input" => values_list.clone(),
                "expectedOutput" => values_list.clone(),
            }
        ),
        ((
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 9, "column": 13}],
                "message": "This resource already exists",
                "path": ["createProgramSpec"],
            })]
        ))
    );

    // Error - invalid values
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "hardwareSpecId" => InputValue::scalar(hw_spec.id.to_string()),
                "name" => InputValue::scalar(""),
                "description" => InputValue::scalar("description!"),
                // TODO use invalid values here once the DB validation is working
                "input" => values_list.clone(),
                "expectedOutput" => values_list.clone(),
            }
        ),
        ((
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 9, "column": 13}],
                "message": "Input validation error(s)",
                "path": ["createProgramSpec"],
                "extensions": {
                    "name": [{"min": "1", "value": "\"\""}],
                }
            })]
        ))
    );
}

#[test]
fn test_update_program_spec() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.context.get_db_conn().unwrap();

    let hw_spec = new_hardware_spec("HW 1").create(conn);
    // We'll test collisions against this
    new_program_spec("Program 1", hw_spec.id).create(conn);
    // This is the one we'll actually be modifying
    let program_spec = new_program_spec("Program 2", hw_spec.id).create(conn);
    let query = r#"
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

    // Success - partial modification, make sure specified fields keep their old
    // value
    assert_eq!(
        runner.query(
            query,
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

    let values_list: InputValue = InputValue::list(
        [1, 2, 3].iter().map(|v| InputValue::scalar(*v)).collect(),
    );

    // Success - modify all fields
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "id" => InputValue::scalar(program_spec.id.to_string()),
                "name" => InputValue::scalar("Program 22"),
                "description" => InputValue::scalar("new description!"),
                "input" => values_list.clone(),
                "expectedOutput" => values_list.clone(),
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

    // Success-ish - invalid ID means just return null
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "id" => InputValue::scalar("bad"),
                "name" => InputValue::scalar("Program 3"),
            }
        ),
        ((
            json!({
                "updateProgramSpec": {
                    "programSpecEdge": serde_json::Value::Null
                }
            }),
            vec![]
        ))
    );

    // Error - modifying no fields
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "id" => InputValue::scalar(program_spec.id.to_string()),
            }
        ),
        ((
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 9, "column": 13}],
                "message": "No fields were given to update",
                "path": ["updateProgramSpec"],
            })]
        ))
    );

    // Error - duplicate name
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "id" => InputValue::scalar(program_spec.id.to_string()),
                "name" => InputValue::scalar("Program 1"),
            }
        ),
        ((
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 9, "column": 13}],
                "message": "This resource already exists",
                "path": ["updateProgramSpec"],
            })]
        ))
    );

    // Error - invalid values
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "id" => InputValue::scalar(program_spec.id.to_string()),
                "name" => InputValue::scalar(""),
            }
        ),
        ((
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 9, "column": 13}],
                "message": "Input validation error(s)",
                "path": ["updateProgramSpec"],
                "extensions": {
                    "name": [{"min": "1", "value": "\"\""}],
                }
            })]
        ))
    );
}

#[test]
fn test_create_user_program() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.context.get_db_conn().unwrap();

    let user_id = new_user("user1").create(conn).id;
    let program_spec_id =
        new_program_spec("prog1", new_hardware_spec("hw1").create(conn).id)
            .create(conn)
            .id;
    // We'll test collisions against this
    models::NewUserProgram {
        user_id,
        program_spec_id,
        file_name: "existing.gdlk",
        source_code: "READ RX0",
    }
    .create(conn);
    let query = r#"
        mutation CreateUserProgramMutation(
            $programSpecId: ID!,
            $fileName: String!,
            $sourceCode: String,
        ) {
            createUserProgram(input: {
                programSpecId: $programSpecId,
                fileName: $fileName,
                sourceCode: $sourceCode,
            }) {
                userProgramEdge {
                    node {
                        fileName
                        sourceCode
                        user {
                            username
                        }
                        programSpec {
                            slug
                        }
                    }
                }
            }
        }
    "#;

    // Known user+program spec combo, with a new file name
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "programSpecId" => InputValue::scalar(program_spec_id.to_string()),
                "fileName" => InputValue::scalar("new.gdlk"),
                "sourceCode" => InputValue::scalar("READ RX0"),
            }
        ),
        (
            json!({
                "createUserProgram": {
                    "userProgramEdge": {
                        "node": {
                            "fileName": "new.gdlk",
                            "sourceCode": "READ RX0",
                            "user": {
                                "username": "user1"
                            },
                            "programSpec": {
                                "slug": "prog1"
                            },
                        }
                    }
                }
            }),
            vec![]
        )
    );

    // Error - Known user+program spec combo, collides with an existing solution
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                        "userId" => InputValue::scalar(user_id.to_string()),
                        "programSpecId" =>
            InputValue::scalar(program_spec_id.to_string()),
            "fileName" => InputValue::scalar("existing.gdlk"),         }
        ),
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 7, "column": 13}],
                "message": "This resource already exists",
                "path": ["createUserProgram"],
            })]
        )
    );

    // Error - Unknown user+program spec combo
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "userId" => InputValue::scalar(user_id.to_string()),
                "programSpecId" => InputValue::scalar("garbage"),
                "fileName" => InputValue::scalar("new.gdlk"),
            }
        ),
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 7, "column": 13}],
                "message": "Not found",
                "path": ["createUserProgram"],
            })]
        )
    );

    // Error - Invalid file name
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "userId" => InputValue::scalar(user_id.to_string()),
                "programSpecId" => InputValue::scalar("garbage"),
                "fileName" => InputValue::scalar(""),
            }
        ),
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 7, "column": 13}],
                "message": "Input validation error(s)",
                "path": ["createUserProgram"],
                "extensions": {
                    "file_name": [{"min": "1", "value": "\"\""}]
                }
            })]
        )
    );
}

#[test]
fn test_update_user_program() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.context.get_db_conn().unwrap();

    // Initialize data
    let user_id = new_user("user1").create(conn).id;
    let program_spec_id =
        new_program_spec("prog1", new_hardware_spec("hw1").create(conn).id)
            .create(conn)
            .id;
    let user_program = models::NewUserProgram {
        user_id,
        program_spec_id,
        file_name: "existing.gdlk",
        source_code: "READ RX0",
    }
    .create(conn);
    // Use this to test collisions
    models::NewUserProgram {
        user_id,
        program_spec_id,
        file_name: "existing2.gdlk",
        source_code: "READ RX0",
    }
    .create(conn);

    let query = r#"
        mutation UpdateUserProgramMutation(
            $id: ID!,
            $fileName: String,
            $sourceCode: String,
        ) {
            updateUserProgram(input: {
                id: $id,
                fileName: $fileName,
                sourceCode: $sourceCode,
            }) {
                userProgramEdge {
                    node {
                        id
                        fileName
                        sourceCode
                        user {
                            username
                        }
                        programSpec {
                            slug
                        }
                    }
                }
            }
        }
    "#;

    // Success - Known user program, with a new file name and source code
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "id" => InputValue::scalar(user_program.id.to_string()),
                "fileName" => InputValue::scalar("new.gdlk"),
                "sourceCode" => InputValue::scalar("WRITE RX0"),
            }
        ),
        (
            json!({
                "updateUserProgram": {
                    "userProgramEdge": {
                        "node": {
                            "id": (user_program.id.to_string()),
                            "fileName": "new.gdlk",
                            "sourceCode": "WRITE RX0",
                            "user": {
                                "username": "user1"
                            },
                            "programSpec": {
                                "slug": "prog1"
                            },
                        }
                    }
                }
            }),
            vec![]
        )
    );

    // Success-ish - Unknown user program, returns null
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "id" => InputValue::scalar("bogus".to_string()),
                "fileName" => InputValue::scalar("new.gdlk"),
                "sourceCode" => InputValue::scalar("WRITE RX0"),
            }
        ),
        (
            json!({
                "updateUserProgram": {
                    "userProgramEdge": serde_json::Value::Null
                }
            }),
            vec![]
        )
    );

    // Error - No update fields specified
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "id" => InputValue::scalar("bogus".to_string()),
            }
        ),
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 7, "column": 13}],
                "message": "No fields were given to update",
                "path": ["updateUserProgram"],
            })]
        )
    );

    // Error - Known user program, rename to collide with another program
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "id" => InputValue::scalar(user_program.id.to_string()),
                "fileName" => InputValue::scalar("existing2.gdlk"),
                "sourceCode" => InputValue::scalar("WRITE RX0"),
            }
        ),
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 7, "column": 13}],
                "message": "This resource already exists",
                "path": ["updateUserProgram"],
            })]
        )
    );

    // Error - Known user program, but the target file name is invalid
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "id" => InputValue::scalar(user_program.id.to_string()),
                "fileName" => InputValue::scalar(""),
            }
        ),
        (
            serde_json::Value::Null,
            vec![json!({
                "locations": [{"line": 7, "column": 13}],
                "message": "Input validation error(s)",
                "path": ["updateUserProgram"],
                "extensions": {
                    "file_name": [{"min": "1", "value": "\"\""}]
                }
            })]
        )
    );
}

#[test]
fn test_delete_user_program() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.context.get_db_conn().unwrap();

    let user_program_id = models::NewUserProgram {
        user_id: new_user("user1").create(conn).id,
        program_spec_id: new_program_spec(
            "prog1",
            new_hardware_spec("hw1").create(conn).id,
        )
        .create(conn)
        .id,
        file_name: "existing.gdlk",
        source_code: "READ RX0",
    }
    .create(conn)
    .id;
    let query = r#"
        mutation DeleteUserProgramMutation($id: ID!) {
            deleteUserProgram(input: { userProgramId: $id }) {
                deletedId
            }
        }
    "#;

    // Known row
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "id" => InputValue::scalar(user_program_id.to_string()),
            }
        ),
        (
            json!({
                "deleteUserProgram": {
                    "deletedId": (user_program_id.to_string())
                }
            }),
            vec![]
        )
    );

    // Not in DB anymore
    assert_eq!(
        user_programs::table
            .find(user_program_id)
            .get_result::<models::UserProgram>(conn)
            .optional()
            .unwrap(),
        None
    );

    // Deleting again gives a null result
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "id" => InputValue::scalar(user_program_id.to_string()),
            }
        ),
        (
            json!({
                "deleteUserProgram": {
                    "deletedId": serde_json::Value::Null
                }
            }),
            vec![]
        )
    );
}
