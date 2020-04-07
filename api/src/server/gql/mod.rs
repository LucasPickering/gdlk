//! The root configuration for GraphQL/Juniper stuff. Only the top-level query
//! and mutation live here, plus some general types that are exposed in the
//! API. Utility types/functions live in `internal`. Specific models live in
//! their own files.

use crate::{
    error::{ServerError, ServerResult},
    models,
    schema::{hardware_specs, users},
    server::gql::{
        hardware::{
            HardwareSpecConnection, HardwareSpecEdge, HardwareSpecNode,
        },
        program::{
            ProgramSpecConnection, ProgramSpecEdge, ProgramSpecNode,
            UserProgramConnection, UserProgramEdge, UserProgramNode,
        },
        user::UserNode,
    },
    util::{Pool, PooledConnection},
};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use gdlk::Valid;
use juniper::EmptyMutation;
use juniper_from_schema::graphql_schema_from_file;
use std::sync::Arc;
use validator::{Validate, ValidationError, ValidationErrors};

mod hardware;
mod internal;
mod program;
mod user;

graphql_schema_from_file!("schema.graphql", error_type: ServerError);

pub struct Context {
    pub pool: Arc<Pool>,
}

impl Context {
    pub fn get_db_conn(&self) -> Result<PooledConnection, ServerError> {
        Ok(self.pool.get()?)
    }
}

// To make our context usable by Juniper, we have to implement a marker trait.
impl juniper::Context for Context {}

/// The top-level query object.
pub struct Query;

impl QueryFields for Query {
    /// Get a node of any type by UUID.
    fn field_node(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, Node, Walked>,
        id: juniper::ID,
    ) -> ServerResult<Option<Node>> {
        internal::get_by_id_from_all_types(&executor.context(), &id)
    }

    fn field_user(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, UserNode, Walked>,
        username: String,
    ) -> ServerResult<Option<UserNode>> {
        Ok(users::table
            .filter(models::User::with_username(&username))
            .get_result::<models::User>(&executor.context().get_db_conn()?)
            .optional()?
            .map(UserNode::from))
    }

    fn field_hardware_spec(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, HardwareSpecNode, Walked>,
        slug: String,
    ) -> ServerResult<Option<HardwareSpecNode>> {
        Ok(hardware_specs::table
            .filter(hardware_specs::dsl::slug.eq(&slug))
            .get_result::<models::HardwareSpec>(
                &executor.context().get_db_conn()?,
            )
            .optional()?
            .map(HardwareSpecNode::from))
    }

    fn field_hardware_specs(
        &self,
        _executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, HardwareSpecConnection, Walked>,
        first: Option<i32>,
        after: Option<Cursor>,
    ) -> ServerResult<HardwareSpecConnection> {
        HardwareSpecConnection::new(first, after)
    }
}

pub type GqlSchema = juniper::RootNode<'static, Query, EmptyMutation<Context>>;

pub fn create_gql_schema() -> GqlSchema {
    GqlSchema::new(Query, EmptyMutation::new())
}

impl Cursor {
    fn from_index(index: i32) -> Self {
        // TODO use base64
        Self(format!("{}", index))
    }

    fn to_index(&self) -> ServerResult<i32> {
        // TODO use base64
        Ok(self.0.parse()?)
    }
}

impl Validate for Cursor {
    fn validate(&self) -> Result<(), ValidationErrors> {
        // we have to implement this manually because the struct definition
        // is auto-generated, so we can't put more macros on it.
        let mut error: ValidationError = match self.to_index() {
            Ok(index) if index >= 0 => {
                return Ok(());
            }
            _ => ValidationError::new("cursor"),
        };
        error.add_param("cursor".into(), &self.0);

        // This error is kinda nasty but way she goes
        let mut errors: ValidationErrors = ValidationErrors::new();
        errors.add("__all__", error);
        Err(errors)
    }
}

/// Helper type to handle pagination params for Connection types. Right now this
/// only supports forward pagination, but in the future we may want to support
/// backwards pagination too. See the Relay Connections spec:
/// https://facebook.github.io/relay/graphql/connections.htm#sec-Forward-pagination-arguments
#[derive(Clone, Debug, Validate)]
pub struct ConnectionPageParams {
    #[validate(range(min = 0))]
    first: Option<i32>,
    #[validate]
    after: Option<Cursor>,
}

impl ConnectionPageParams {
    fn new(
        first: Option<i32>,
        after: Option<Cursor>,
    ) -> Result<Valid<Self>, ValidationErrors> {
        Ok(Valid::validate(Self { first, after })?)
    }

    /// Convert the pagination parameters into an index offset. This determines
    /// how many rows we want to skip before the first returned row. Can be
    /// used directly in a SQL `OFFSET` clause.
    fn offset(&self) -> i32 {
        match &self.after {
            None => 0,
            Some(cursor) => {
                // This unwrap is safe, since we validate in the constructor.
                // Unfortunately there's no good way to leverage the type system
                // to get around this.
                cursor.to_index().unwrap() + 1
            }
        }
    }

    /// Get the maximum number of rows to return. If `None`, as many rows as
    /// possible will be returned.
    fn limit(&self) -> Option<i32> {
        // if we ever want to support reverse pagination, this will be useful
        self.first
    }
}

/// GQL type to display information about a page of data. See the Relay
/// Connections spec: https://facebook.github.io/relay/graphql/connections.htm#sec-undefined.PageInfo
#[derive(Clone, Debug, PartialEq)]
pub struct PageInfo {
    start_cursor: Option<Cursor>,
    end_cursor: Option<Cursor>,
    has_previous_page: bool,
    has_next_page: bool,
}

impl PageInfo {
    /// Calculate page metadata based on the input pagination params, plus the
    /// total number of rows matching the query. As long as the query properly
    /// adhered to the input pagination params, then this will accurately
    /// determine the page metdata. `total_count` should _include_ rows that
    /// do not appear in the page, but _exclude_ rows that do not match any
    /// filters that may have been applied.
    pub fn from_page_params(
        page_params: &Valid<ConnectionPageParams>,
        total_count: i32,
    ) -> Self {
        let offset: i32 = page_params.offset();
        let limit_opt: Option<i32> = page_params.limit();

        let start_index = offset;
        let end_index = match limit_opt {
            None => total_count - 1,
            Some(limit) => i32::min(offset + limit, total_count) - 1,
        };

        let (start_cursor, end_cursor) = if end_index < start_index {
            // Data is empty
            (None, None)
        } else {
            (
                Some(Cursor::from_index(start_index)),
                Some(Cursor::from_index(end_index)),
            )
        };

        Self {
            start_cursor,
            end_cursor,
            has_previous_page: 0 < start_index && 0 < total_count,
            has_next_page: end_index + 1 < total_count,
        }
    }
}

impl PageInfoFields for PageInfo {
    fn field_start_cursor(
        &self,
        _executor: &juniper::Executor<'_, Context>,
    ) -> &Option<Cursor> {
        &self.start_cursor
    }

    fn field_end_cursor(
        &self,
        _executor: &juniper::Executor<'_, Context>,
    ) -> &Option<Cursor> {
        &self.end_cursor
    }

    fn field_has_previous_page(
        &self,
        _executor: &juniper::Executor<'_, Context>,
    ) -> bool {
        self.has_previous_page
    }

    fn field_has_next_page(
        &self,
        _executor: &juniper::Executor<'_, Context>,
    ) -> bool {
        self.has_next_page
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_page_params_validation() {
        ConnectionPageParams::new(Some(-1), None).unwrap_err();
        ConnectionPageParams::new(None, Some(Cursor("-1".into()))).unwrap_err();
        ConnectionPageParams::new(None, Some(Cursor("garbage".into())))
            .unwrap_err();
    }

    #[test]
    fn test_connection_page_params_offset_limit() {
        let mut params: Valid<ConnectionPageParams>;

        params = ConnectionPageParams::new(None, None).unwrap();
        assert_eq!(params.offset(), 0);
        assert_eq!(params.limit(), None);

        params =
            ConnectionPageParams::new(Some(0), Some(Cursor::from_index(0)))
                .unwrap();
        assert_eq!(params.offset(), 1);
        assert_eq!(params.limit(), Some(0));

        params =
            ConnectionPageParams::new(Some(1), Some(Cursor::from_index(1)))
                .unwrap();
        assert_eq!(params.offset(), 2);
        assert_eq!(params.limit(), Some(1));
    }

    #[test]
    fn test_page_info_from_page_params_no_params() {
        assert_eq!(
            PageInfo::from_page_params(
                &ConnectionPageParams::new(None, None).unwrap(),
                0
            ),
            PageInfo {
                start_cursor: None,
                end_cursor: None,
                has_previous_page: false,
                has_next_page: false,
            }
        );

        assert_eq!(
            PageInfo::from_page_params(
                &ConnectionPageParams::new(None, None).unwrap(),
                1
            ),
            PageInfo {
                start_cursor: Some(Cursor::from_index(0)),
                end_cursor: Some(Cursor::from_index(0)),
                has_previous_page: false,
                has_next_page: false,
            }
        );

        assert_eq!(
            PageInfo::from_page_params(
                &ConnectionPageParams::new(None, None).unwrap(),
                10
            ),
            PageInfo {
                start_cursor: Some(Cursor::from_index(0)),
                end_cursor: Some(Cursor::from_index(9)),
                has_previous_page: false,
                has_next_page: false,
            }
        );
    }

    #[test]
    fn test_page_info_from_page_params_with_first() {
        // first = 0
        assert_eq!(
            PageInfo::from_page_params(
                &ConnectionPageParams::new(Some(0), None).unwrap(),
                0
            ),
            PageInfo {
                start_cursor: None,
                end_cursor: None,
                has_previous_page: false,
                has_next_page: false,
            }
        );
        assert_eq!(
            PageInfo::from_page_params(
                &ConnectionPageParams::new(Some(0), None).unwrap(),
                1
            ),
            PageInfo {
                start_cursor: None,
                end_cursor: None,
                has_previous_page: false,
                has_next_page: true,
            }
        );

        // first > 0, total_count = 0
        assert_eq!(
            PageInfo::from_page_params(
                &ConnectionPageParams::new(Some(5), None).unwrap(),
                0
            ),
            PageInfo {
                start_cursor: None,
                end_cursor: None,
                has_previous_page: false,
                has_next_page: false,
            }
        );

        // first > total_count > 0
        assert_eq!(
            PageInfo::from_page_params(
                &ConnectionPageParams::new(Some(5), None).unwrap(),
                3
            ),
            PageInfo {
                start_cursor: Some(Cursor::from_index(0)),
                end_cursor: Some(Cursor::from_index(2)),
                has_previous_page: false,
                has_next_page: false,
            }
        );

        // first = total_count
        assert_eq!(
            PageInfo::from_page_params(
                &ConnectionPageParams::new(Some(5), None).unwrap(),
                5
            ),
            PageInfo {
                start_cursor: Some(Cursor::from_index(0)),
                end_cursor: Some(Cursor::from_index(4)),
                has_previous_page: false,
                has_next_page: false,
            }
        );

        // first < total_count
        assert_eq!(
            PageInfo::from_page_params(
                &ConnectionPageParams::new(Some(5), None).unwrap(),
                10
            ),
            PageInfo {
                start_cursor: Some(Cursor::from_index(0)),
                end_cursor: Some(Cursor::from_index(4)),
                has_previous_page: false,
                has_next_page: true,
            }
        );
    }

    #[test]
    fn test_page_info_from_page_params_with_after() {
        // total_count = 0
        assert_eq!(
            PageInfo::from_page_params(
                &ConnectionPageParams::new(None, Some(Cursor::from_index(0)))
                    .unwrap(),
                0
            ),
            PageInfo {
                start_cursor: None,
                end_cursor: None,
                has_previous_page: false,
                has_next_page: false,
            }
        );

        // after = last index
        assert_eq!(
            PageInfo::from_page_params(
                &ConnectionPageParams::new(None, Some(Cursor::from_index(0)))
                    .unwrap(),
                1
            ),
            PageInfo {
                start_cursor: None,
                end_cursor: None,
                has_previous_page: true,
                has_next_page: false,
            }
        );

        // requested last element
        assert_eq!(
            PageInfo::from_page_params(
                &ConnectionPageParams::new(None, Some(Cursor::from_index(0)))
                    .unwrap(),
                2
            ),
            PageInfo {
                start_cursor: Some(Cursor::from_index(1)),
                end_cursor: Some(Cursor::from_index(1)),
                has_previous_page: true,
                has_next_page: false,
            }
        );

        // multiple elements returned
        assert_eq!(
            PageInfo::from_page_params(
                &ConnectionPageParams::new(None, Some(Cursor::from_index(0)))
                    .unwrap(),
                3
            ),
            PageInfo {
                start_cursor: Some(Cursor::from_index(1)),
                end_cursor: Some(Cursor::from_index(2)),
                has_previous_page: true,
                has_next_page: false,
            }
        );
    }

    #[test]
    fn test_page_info_from_page_params_with_both() {
        // zeroes
        assert_eq!(
            PageInfo::from_page_params(
                &ConnectionPageParams::new(
                    Some(0),
                    Some(Cursor::from_index(0)),
                )
                .unwrap(),
                0
            ),
            PageInfo {
                start_cursor: None,
                end_cursor: None,
                has_previous_page: false,
                has_next_page: false,
            }
        );

        // first = 0
        assert_eq!(
            PageInfo::from_page_params(
                &ConnectionPageParams::new(
                    Some(0),
                    Some(Cursor::from_index(0)),
                )
                .unwrap(),
                1
            ),
            PageInfo {
                start_cursor: None,
                end_cursor: None,
                has_previous_page: true,
                has_next_page: false,
            }
        );

        // requested amount > returned amount
        assert_eq!(
            PageInfo::from_page_params(
                &ConnectionPageParams::new(
                    Some(10),
                    Some(Cursor::from_index(0)),
                )
                .unwrap(),
                2
            ),
            PageInfo {
                start_cursor: Some(Cursor::from_index(1)),
                end_cursor: Some(Cursor::from_index(1)),
                has_previous_page: true,
                has_next_page: false,
            }
        );

        // requested amount = available amount
        assert_eq!(
            PageInfo::from_page_params(
                &ConnectionPageParams::new(
                    Some(2),
                    Some(Cursor::from_index(0)),
                )
                .unwrap(),
                3
            ),
            PageInfo {
                start_cursor: Some(Cursor::from_index(1)),
                end_cursor: Some(Cursor::from_index(2)),
                has_previous_page: true,
                has_next_page: false,
            }
        );

        // requested amount < available amount
        assert_eq!(
            PageInfo::from_page_params(
                &ConnectionPageParams::new(
                    Some(10),
                    Some(Cursor::from_index(10)),
                )
                .unwrap(),
                30
            ),
            PageInfo {
                start_cursor: Some(Cursor::from_index(11)),
                end_cursor: Some(Cursor::from_index(20)),
                has_previous_page: true,
                has_next_page: true,
            }
        );
    }
}
