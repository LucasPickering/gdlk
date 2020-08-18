use crate::{
    error::{ClientError, ResponseError, ResponseResult},
    models,
    schema::users,
    server::gql::{
        internal::{GenericEdge, NodeType},
        AuthStatusFields, Cursor, InitializeUserPayloadFields, UserEdgeFields,
        UserNodeFields,
    },
    util,
    views::RequestContext,
};
use diesel::{PgConnection, QueryDsl, QueryResult, RunQueryDsl};
use juniper::ID;
use juniper_from_schema::{QueryTrail, Walked};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct UserNode {
    pub user: models::User,
}

impl From<models::User> for UserNode {
    fn from(model: models::User) -> Self {
        Self { user: model }
    }
}

impl NodeType for UserNode {
    type Model = models::User;

    fn find(conn: &PgConnection, id: Uuid) -> QueryResult<Self::Model> {
        users::table.find(id).get_result(conn)
    }
}

impl UserNodeFields for UserNode {
    fn field_id(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
    ) -> ID {
        util::uuid_to_gql_id(self.user.id)
    }

    fn field_username(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
    ) -> &String {
        &self.user.username
    }
}

pub type UserEdge = GenericEdge<UserNode>;

impl UserEdgeFields for UserEdge {
    fn field_node(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
        _trail: &QueryTrail<'_, UserNode, Walked>,
    ) -> &UserNode {
        self.node()
    }

    fn field_cursor(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
    ) -> &Cursor {
        self.cursor()
    }
}

/// A simple wrapper for a few fields that pertain to the requesting user's
/// authentication status.
pub struct AuthStatus();

impl AuthStatusFields for AuthStatus {
    fn field_authenticated(
        &self,
        executor: &juniper::Executor<'_, RequestContext>,
    ) -> bool {
        executor.context().user_context.is_some()
    }

    fn field_user_created(
        &self,
        executor: &juniper::Executor<'_, RequestContext>,
    ) -> bool {
        executor.context().user().is_ok()
    }

    fn field_user(
        &self,
        executor: &juniper::Executor<'_, RequestContext>,
        _trail: &QueryTrail<'_, UserNode, Walked>,
    ) -> ResponseResult<Option<UserNode>> {
        match executor.context().user() {
            Ok(user) => Ok(Some(user.clone().into_model().into())),
            // User isn't authed or hasn't finished setup
            Err(ResponseError::Client {
                source: ClientError::Unauthenticated,
                ..
            }) => Ok(None),
            // This shouldn't be possible
            Err(err) => Err(err),
        }
    }
}

pub struct InitializeUserPayload {
    pub user: models::User,
}

impl InitializeUserPayloadFields for InitializeUserPayload {
    fn field_user_edge(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
        _trail: &QueryTrail<'_, UserEdge, Walked>,
    ) -> UserEdge {
        GenericEdge::from_db_row(self.user.clone(), 0)
    }
}
