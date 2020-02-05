//! The root configuration for GraphQL/Juniper stuff. Not all of the GraphQL
//! models live here, the VFS stuff is in its own module. We may want to put all
//! GQL stuff in one module at some point, but not currently.

use crate::{
    error::Result,
    models::User,
    util::Pool,
    vfs::{NodeMutation, NodeReference, VirtualFileSystem},
};
use diesel::RunQueryDsl;
use juniper::FieldResult;
use std::{rc::Rc, sync::Arc};

pub struct GqlContext {
    pub pool: Arc<Pool>,
}

// To make our context usable by Juniper, we have to implement a marker trait.
impl juniper::Context for GqlContext {}

/// Helper to get a node for a context, user, and path. This is used for both
/// the query and mutation handlers, so we factor out the common code here.
fn get_fs_node(
    context: &GqlContext,
    username: String,
    path: String,
) -> Result<NodeReference> {
    // temporary way to get user -- just have the caller specify username
    let user: User =
        User::filter_by_username(&username).get_result(&context.pool.get()?)?;

    let fs =
        VirtualFileSystem::new(Rc::new(context.pool.get()?), Rc::new(user));
    fs.get_node(&path)
}

/// The top-level query object
pub struct RootQuery;

#[juniper::object(Context = GqlContext)]
impl RootQuery {
    /// A file system read operation.
    fn fs_node(
        context: &GqlContext,
        username: String,
        path: String,
    ) -> FieldResult<NodeReference> {
        Ok(get_fs_node(context, username, path)?)
    }
}

/// The top-level mutation object
pub struct RootMutation;

#[juniper::object(Context = GqlContext)]
impl RootMutation {
    /// A file system write operation.
    fn fs_node(
        context: &GqlContext,
        username: String,
        path: String,
    ) -> FieldResult<NodeMutation> {
        Ok(NodeMutation::new(get_fs_node(context, username, path)?))
    }
}

pub type GqlSchema = juniper::RootNode<'static, RootQuery, RootMutation>;

pub fn create_gql_schema() -> GqlSchema {
    GqlSchema::new(RootQuery {}, RootMutation {})
}
