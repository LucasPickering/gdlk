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
use juniper::{
    graphql_value, DefaultScalarValue, ExecutionError, InputValue, Variables,
};
use maplit::hashmap;
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

fn new_hardware_spec(slug: &str) -> models::NewHardwareSpec {
    models::NewHardwareSpec {
        slug,
        num_registers: 1,
        num_stacks: 0,
        max_stack_length: 0,
    }
}

fn new_program_spec(
    slug: &str,
    hardware_spec_id: Uuid,
) -> models::NewProgramSpec {
    models::NewProgramSpec {
        slug,
        hardware_spec_id,
        input: Vec::new(),
        expected_output: Vec::new(),
    }
}

/// Wrap a vec into a GraphQL value. This will wrap each child in the value,
/// then wrap the vec itself.
fn to_gql_list<T: Into<juniper::Value>>(list: Vec<T>) -> juniper::Value {
    juniper::Value::List(list.into_iter().map(|v| v.into()).collect())
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
    ) -> (
        juniper::Value<DefaultScalarValue>,
        Vec<ExecutionError<DefaultScalarValue>>,
    ) {
        // Map &strs to Strings
        let converted_vars = vars
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect::<Variables>();

        juniper::execute(
            query,
            None,
            &self.schema,
            &converted_vars,
            &self.context,
        )
        .unwrap()
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
                graphql_value!({
                    "node": {
                        "id": (id.to_string()),
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
                graphql_value!({
                    "node": None,
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
            graphql_value!({
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
        (graphql_value!({ "user": None }), vec![])
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
            graphql_value!({
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
        (graphql_value!({ "hardwareSpec": None }), vec![])
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
            graphql_value!({
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
            graphql_value!({
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
            graphql_value!({
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
            graphql_value!({
                "hardwareSpec": {
                    "programSpec": None,
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
        slug: "prog1",
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
            graphql_value!({
                "hardwareSpec": {
                    "programSpec": {
                        "id": (program_spec_id.to_string()),
                        "slug": "prog1",
                        "input": (to_gql_list(vec![1, 2, 3])),
                        "expectedOutput": (to_gql_list(vec![1, 2, 3])),
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
            graphql_value!({
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
fn test_save_user_program() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.context.get_db_conn().unwrap();

    let user_id = new_user("user1").create(conn).id;
    let program_spec_id =
        new_program_spec("prog1", new_hardware_spec("hw1").create(conn).id)
            .create(conn)
            .id;
    models::NewUserProgram {
        user_id,
        program_spec_id,
        file_name: "existing.gdlk",
        source_code: "READ RX0",
    }
    .create(conn);
    let query = r#"
        mutation SaveUserProgramMutation(
            $programSpecId: ID!,
            $fileName: String!,
            $sourceCode: String!,
        ) {
            saveUserProgram(input: {
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
            graphql_value!({
                "saveUserProgram": {
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

    // Known user+program spec combo, overwriting an existing solution
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "userId" => InputValue::scalar(user_id.to_string()),
                "programSpecId" => InputValue::scalar(program_spec_id.to_string()),
                "fileName" => InputValue::scalar("existing.gdlk"),
                "sourceCode" => InputValue::scalar("WRITE RX0"),
            }
        ),
        (
            graphql_value!({
                "saveUserProgram": {
                    "userProgramEdge": {
                        "node":{
                            "fileName": "existing.gdlk",
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

    // Unknown user+program spec combo
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "userId" => InputValue::scalar(user_id.to_string()),
                "programSpecId" => InputValue::scalar("garbage"),
                "fileName" => InputValue::scalar("new.gdlk"),
                "sourceCode" => InputValue::scalar("READ RX0"),
            }
        ),
        (
            graphql_value!({
                "saveUserProgram": {
                    "userProgramEdge": None
                }
            }),
            vec![]
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
            graphql_value!({
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
            graphql_value!({
                "deleteUserProgram": {
                    "deletedId": None
                }
            }),
            vec![]
        )
    );
}
