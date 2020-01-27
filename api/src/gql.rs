//! The root configuration for GraphQL/Juniper stuff. Not all of the GraphQL
//! models live here, the VFS stuff is in its own module. We may want to put all
//! GQL stuff in one module at some point, but not currently.

use crate::{
    models::User,
    util::Pool,
    vfs::{Node, VirtualFileSystem},
};
use diesel::RunQueryDsl;
use juniper::FieldResult;
use std::{rc::Rc, sync::Arc};

pub struct GqlContext {
    pub pool: Arc<Pool>,
}

// To make our context usable by Juniper, we have to implement a marker trait.
impl juniper::Context for GqlContext {}

pub struct RootQuery;

#[juniper::object(Context = GqlContext)]
impl RootQuery {
    fn api_version() -> &str {
        "1.0"
    }

    fn fs_node(
        context: &GqlContext,
        username: String,
        path: String,
    ) -> FieldResult<Node> {
        // temporary way to get user -- just have the caller specify username
        let user: User = User::filter_by_username(&username)
            .get_result(&context.pool.get()?)?;

        let fs =
            VirtualFileSystem::new(Rc::new(context.pool.get()?), Rc::new(user));
        Ok(fs.node(&path)?)
    }
}

pub struct RootMutation;

#[juniper::object(Context = GqlContext)]
impl RootMutation {}

pub type GqlSchema = juniper::RootNode<'static, RootQuery, RootMutation>;

pub fn create_gql_schema() -> GqlSchema {
    GqlSchema::new(RootQuery {}, RootMutation {})
}
