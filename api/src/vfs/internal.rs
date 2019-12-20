//! This module holds types internal to this module. None of these types should
//! be exported outside this module. These types are separated just for
//! readability.

use crate::{
    error::{Result, ServerError},
    vfs::{NodeMetadata, NodePermissions, NodeType},
};
use diesel::PgConnection;
use std::{collections::HashMap, fmt::Debug};

/// A definition for a path segment in the virtual tree. The definition should
/// be associated with a singular virtual node.
#[derive(Copy, Clone, Debug)]
pub enum SegmentSpec {
    /// The associated virtual node is referenced by a particular static
    /// string. This virtual node corresponds to, at most, 1 physical node.
    Fixed(&'static str),
    /// The associated virtual node can be referenced by any number of path
    /// segments. Any path segment will match this spec, and further operations
    /// by the node handler will be needed to determine if the physical node
    /// exists. The string here refers to the variable name under which the
    /// path segment gets store.
    Variable(&'static str),
}

impl SegmentSpec {
    /// Checks if the given path segment matches this spec. Fixed segment specs
    /// require an exact string match, while variable specs will match any
    /// segment.
    pub fn matches(&self, path_segment: &str) -> bool {
        match self {
            Self::Fixed(expected_segment) => *expected_segment == path_segment,
            // At some point we may want to make the variables allow
            // regexes, in which case those would be applied here
            Self::Variable(_) => true,
        }
    }
}

/// The context needed in order to serve an fs node. This context is available
/// to any fs operation. The variables are populated by the contents of the
/// path. For any particular node handler, the entries in this map are fixed.
///
/// NOTE: There will NOT be a variable defined for the last segment in the path,
/// i.e. the segment being operated on. This is because the variable would just
/// be a duplicate of that value, which gets passed around anyway (see example).
///
/// Ideally, instead of a map we would have fixed identifiers, but that requires
/// macro wizardry that I just don't want to write. Example:
///
/// - Path definition: "/hardware/<hw_spec_slug>/programs/<program_spec_slug>"
/// - Path: "/hardware/hw1/programs/prog1"
/// - Variables:
///     - "hardware_spec_slug":"hw1"
pub struct Context<'a> {
    pub db_conn: &'a PgConnection,
    pub variables: HashMap<&'a str, &'a str>,
}

impl<'a> Context<'a> {
    /// Helper for accessing path variables by name. All variable names are
    /// fixed at compile time, and all variables must be present in a path in
    /// order for it to match, so this should always succeed. A missing variable
    /// indicates a bug, hence why it panics in that case.
    pub fn get_var(&self, var_name: &'static str) -> &str {
        self.variables.get(var_name).expect("Unknown path variable")
    }
}

/// A definition of functionality for serving physical nodes for a particular
/// virtual node. Each virtual node in the tree needs an implementation of this
/// trait. Some simple nodes _may_ be able to share implementations, but for the
/// most part each virtual node will need its own corresponding handler struct.
/// In general, the structs won't hold any data, they will just be there to hold
/// the functions needed for that virtual node.
pub trait VirtualNodeHandler: Debug + Sync {
    /// Checks if a physical node exists at the given path segment for this
    /// virtual node.
    fn exists(&self, _context: &Context, _path_segment: &str) -> Result<bool> {
        // This default implementation covers fixed virtual nodes, which will
        // generally exist as long as their parent exists. Override this if
        // you have a variable node, or if a fixed node can exist conditionally.
        Ok(true)
    }

    /// Gets the permission for the physical node at the given path segment.
    fn get_permissions(
        &self,
        context: &Context,
        path_segment: &str,
    ) -> Result<NodePermissions>;

    /// Gets the contents of the file at the given path segment.
    fn get_content(
        &self,
        _context: &Context,
        _path_segment: &str,
    ) -> Result<String> {
        // This only needs to be implemented for files. For directories, this
        // should never be called because the Node wrapper checks the node type.
        panic!("Operation not supported for this node type")
    }

    /// Lists all physical nodes that exist for this virtual node. Unlike the
    /// other operations on this trait, this function doesn't take a path
    /// segment, because it doesn't operate on a single physical node.
    fn list_physical_nodes(&self, _context: &Context) -> Result<Vec<String>> {
        // This only needs to be implemented for directories with variable path
        // segments. For files and fixed directories, this should never be
        // called.
        panic!("Operation not supported for this node type")
    }
}

/// A virtual node in the file system. In this context, "virtual" means that
/// the node does not necessarily correspond to a single file or directory.
/// Instead, it dynamically serves many files/directories dynamically, based
/// on input data such as user and certain path variables.
#[derive(Debug)]
pub struct VirtualNode {
    pub path_segment: SegmentSpec,
    pub node_type: NodeType,
    pub handler: &'static dyn VirtualNodeHandler,
    pub children: &'static [VirtualNode],
}

impl VirtualNode {
    pub const fn file(
        path_segment: SegmentSpec,
        handler: &'static impl VirtualNodeHandler,
    ) -> Self {
        VirtualNode {
            path_segment,
            node_type: NodeType::File,
            handler,
            children: &[],
        }
    }

    pub const fn dir(
        path_segment: SegmentSpec,
        handler: &'static impl VirtualNodeHandler,
        children: &'static [Self],
    ) -> Self {
        VirtualNode {
            path_segment,
            node_type: NodeType::Directory,
            handler,
            children,
        }
    }

    pub fn match_child(
        &self,
        path_segment: &str,
    ) -> Option<&'static VirtualNode> {
        self.children
            .iter()
            .find(|child| child.path_segment.matches(path_segment))
    }

    pub fn exists(
        &self,
        context: &Context,
        path_segment: &str,
    ) -> Result<bool> {
        self.handler.exists(context, path_segment)
    }

    pub fn get_metadata(
        &self,
        context: &Context,
        path_segment: &str,
    ) -> Result<NodeMetadata> {
        Ok(NodeMetadata {
            node_type: self.node_type,
            name: path_segment.into(),
            permissions: self.handler.get_permissions(context, path_segment)?,
        })
    }

    pub fn get_content(
        &self,
        context: &Context,
        path_segment: &str,
    ) -> Result<String> {
        match self.node_type {
            NodeType::File => self.handler.get_content(context, path_segment),
            // Can't do this for directories, ya dummy
            NodeType::Directory => Err(ServerError::UnsupportedFileOperation),
        }
    }

    /// List all physical nodes that are children of this virtual node. The path
    /// segment isn't needed for this operation, but it's included in the
    /// signature to match <NodeOperation>.
    pub fn list_children(
        &self,
        context: &Context,
        _path_segment: &str,
    ) -> Result<Vec<NodeMetadata>> {
        match self.node_type {
            NodeType::File => Err(ServerError::UnsupportedFileOperation),
            NodeType::Directory => {
                let mut rv: Vec<NodeMetadata> = Vec::new();
                // Each virtual child could map to 0 or more physical nodes.
                // Fixed children will map to exactly 1, but variable ones need
                // to be listed dynamically (e.g. via a DB lookup) and so could
                // be 0+ physical nodes.
                for child in self.children {
                    match child.path_segment {
                        // For fixed children, just fetch their metadata
                        SegmentSpec::Fixed(child_segment) => {
                            rv.push(child.get_metadata(context, child_segment)?)
                        }
                        // For variable children, we'll need to dynamically
                        // fetch a list of nodes. This looks different for each
                        // node type.
                        SegmentSpec::Variable(_) => rv.extend(
                            child
                                .handler
                                .list_physical_nodes(context)?
                                .iter()
                                .map(|child_segment| {
                                    child.get_metadata(context, child_segment)
                                })
                                // Each element in the iter is a Result,
                                // we
                                // need to collect into one Result
                                .collect::<Result<Vec<NodeMetadata>>>()?,
                        ),
                    }
                }
                Ok(rv)
            }
        }
    }
}
