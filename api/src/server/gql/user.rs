use crate::{
    models,
    schema::users,
    server::gql::{internal::NodeType, Context, UserNodeFields},
    util,
};
use diesel::{PgConnection, QueryDsl, QueryResult, RunQueryDsl};
use juniper::ID;
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
        util::uuid_to_gql_id(&self.user.id)
    }

    fn field_username(
        &self,
        _executor: &juniper::Executor<'_, Context>,
    ) -> &String {
        &self.user.username
    }
}
