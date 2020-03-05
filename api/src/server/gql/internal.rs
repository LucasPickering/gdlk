use crate::{
    error::ServerResult,
    server::gql::{
        hardware::HardwareSpecNode, program::ProgramSpecNode, Context, Cursor,
        Node,
    },
    util,
};
use diesel::{OptionalExtension, PgConnection, QueryResult};
use std::convert::TryFrom;
use uuid::Uuid;

/// Reference to the get_by_id function for all node types.
const ALL_NODES_GET_BY_ID: &[&dyn Fn(
    &PgConnection,
    Uuid,
) -> ServerResult<Option<Node>>] =
    &[&HardwareSpecNode::get_by_id, &ProgramSpecNode::get_by_id];

/// A trait to identify any struct that is a GQL node.
pub trait NodeType: Sized + Into<Node> {
    /// The DB model type associated with this node type.
    type Model;

    /// Convert the underlying model value to a value of this type.
    fn from_model(model: Self::Model) -> Self;

    /// Query the DB table by ID.
    fn find(conn: &PgConnection, id: Uuid) -> QueryResult<Self::Model>;

    /// A wrapper around [Self::find] that converts the result into an
    /// `Option<Node>`, so that it has a uniform output type with all other
    /// node types.
    fn get_by_id(conn: &PgConnection, id: Uuid) -> ServerResult<Option<Node>> {
        Ok(Self::find(conn, id)
            .optional()?
            .map(|model| Self::from_model(model).into()))
    }
}

/// Helper type to handle GQL edge types. Edges consist of a cursor, to locate
/// the edge within a Connection, and an associated node.
pub struct GenericEdge<N: NodeType> {
    node: N,
    cursor: Cursor,
}

impl<N: NodeType> GenericEdge<N> {
    pub fn node(&self) -> &N {
        &self.node
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    /// Convert a list of DB model rows into a list of this type. `offset` is
    /// the index of the first row in the database (with the ordering with which
    /// it was queried). The offset is used to determine the cursor for each
    /// edge.
    pub fn from_db_rows(rows: Vec<N::Model>, offset: i32) -> Vec<Self> {
        rows.into_iter()
            .enumerate()
            .map(|(i, row)| Self {
                node: NodeType::from_model(row),
                cursor: Cursor::from_index(offset + i32::try_from(i).unwrap()),
            })
            .collect()
    }
}

/// Get the node with the given global ID. This queries each DB table that
/// associated with a node type, and finds the row with the matching ID. Since
/// we use UUIDs, this will never match more than one row across all the tables.
pub fn get_by_id_from_all_types(
    context: &Context,
    id: &juniper::ID,
) -> ServerResult<Option<Node>> {
    let conn: &PgConnection = &context.get_db_conn()? as &PgConnection;
    let uuid_id = util::gql_id_to_uuid(id)?;

    // Do one query per node type to fine the node with this ID
    // This isn't the most efficient solution but ¯\_(ツ)_/¯
    for f in ALL_NODES_GET_BY_ID {
        if let Some(node) = f(conn, uuid_id)? {
            return Ok(Some(node));
        }
    }
    Ok(None)
}
