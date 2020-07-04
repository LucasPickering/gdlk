use crate::{
    error::{ResponseError, ResponseResult},
    models,
    schema::users,
    server::gql::{
        internal::NodeType, AuthStatusFields, Context, UserNodeFields,
    },
    util,
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
    fn field_id(&self, _executor: &juniper::Executor<'_, Context>) -> ID {
        util::uuid_to_gql_id(self.user.id)
    }

    fn field_username(
        &self,
        _executor: &juniper::Executor<'_, Context>,
    ) -> &String {
        &self.user.username
    }
}

/// A simple wrapper for a few fields that pertain to the requesting user's
/// authentication status.
pub struct AuthStatus();

impl AuthStatusFields for AuthStatus {
    fn field_authenticated(
        &self,
        executor: &juniper::Executor<'_, Context>,
    ) -> bool {
        executor.context().user_context.is_some()
    }

    fn field_user_created(
        &self,
        executor: &juniper::Executor<'_, Context>,
    ) -> bool {
        executor.context().user_id().is_ok()
    }

    fn field_user(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, UserNode, Walked>,
    ) -> ResponseResult<Option<UserNode>> {
        let context = executor.context();

        match executor.context().user_id() {
            Ok(user_id) => {
                let user: models::User = users::table
                    .find(user_id)
                    .get_result(&context.get_db_conn()?)?;
                Ok(Some(user.into()))
            }
            // User isn't authed or hasn't finished setup
            Err(ResponseError::Unauthenticated) => Ok(None),
            // This shouldn't be possible
            Err(err) => Err(err),
        }
    }
}
