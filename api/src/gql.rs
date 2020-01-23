//! The root configuration for GraphQL/Juniper stuff. Not all of the GraphQL
//! models live here, the VFS stuff is in its own module. We may want to put all
//! GQL stuff in one module at some point, but not currently.

use crate::{
    util::Pool,
    vfs::{Node, VirtualFileSystem},
};
use juniper::FieldResult;
use std::sync::Arc;

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

    fn fs_node(context: &GqlContext, path: String) -> FieldResult<Node> {
        // pool is an Arc, so we can cheaply clone it
        let fs = VirtualFileSystem::new(Arc::new(context.pool.get()?));
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
