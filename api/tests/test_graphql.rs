use diesel::{
    associations::HasTable, query_source::Table, PgConnection, RunQueryDsl,
};
use failure::Fallible;
use gdlk_api::{
    models,
    server::{create_gql_schema, Context, GqlSchema},
    util,
};
use juniper::{
    graphql_value, DefaultScalarValue, ExecutionError, InputValue, Variables,
};
use maplit::hashmap;
use std::{collections::HashMap, sync::Arc};

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
            models::User,
            models::HardwareSpec,
            models::ProgramSpec,
            models::UserProgram,
            // Any new table needs to be added here!
        );
    }
}

#[test]
fn test_field_node() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.context.get_db_conn().unwrap();

    let user: models::User = models::NewUser { username: "user1" }
        .insert()
        .returning(<models::User as HasTable>::Table::all_columns())
        .get_result(conn)
        .unwrap();

    let query = r#"
        query NodeQuery($nodeId: ID!) {
            node(id: $nodeId) {
                id
            }
        }
    "#;

    // Known user
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "nodeId" => InputValue::scalar(user.id.to_string()),
            }
        ),
        (
            graphql_value!({
                "node": {
                    "id": (user.id.to_string()),
                }
            }),
            vec![]
        )
    );

    // Invalid UUID
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "nodeId" => InputValue::scalar("invalid-uuid"),
            }
        ),
        (
            graphql_value!({
                "node": None,
            }),
            vec![]
        )
    );

    // Unknown UUID
    assert_eq!(
        runner.query(
            query,
            hashmap! {
                "nodeId" => InputValue::scalar("1bb9a3a1-0149-4264-a0a7-ff17ac7b8a61"),
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

#[test]
fn test_field_user() {
    let runner = QueryRunner::new().unwrap();
    let conn: &PgConnection = &runner.context.get_db_conn().unwrap();

    let user: models::User = models::NewUser { username: "user1" }
        .insert()
        .returning(<models::User as HasTable>::Table::all_columns())
        .get_result(conn)
        .unwrap();
    let query = r#"
        query UserQuery($username: String!) {
            user(username: $username) {
                id
                username
            }
        }
    "#;

    assert_eq!(
        runner.query(
            query,
            hashmap! { "username" => InputValue::scalar("user1") }
        ),
        (
            graphql_value!({
                "user": {
                    "id": (user.id.to_string()),
                    "username": "user1"
                }
            }),
            vec![]
        )
    );

    assert_eq!(
        runner.query(
            query,
            hashmap! { "username" => InputValue::scalar("unknown_user") }
        ),
        (graphql_value!({ "user": None }), vec![])
    );
}
