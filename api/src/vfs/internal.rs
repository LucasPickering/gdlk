//! This module holds types internal to this module. None of these types should
//! be exported outside this module. These types are separated just for
//! readability.

use crate::{
    error::Result,
    models::User,
    util::PooledConnection,
    vfs::{NodePermissions, NodeType},
};
use diesel::PgConnection;
use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashMap, fmt::Debug, rc::Rc};

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
/// to any fs operation, and are constant for every node in the tree. This uses
/// Rcs because it needs to be clonable without being dependent on references.
/// This allows it to be freely copied between nodes, without any node being
/// reliant on any outside lifetimes.
#[derive(Clone)]
pub struct Context {
    db_conn: Rc<PooledConnection>,
    // It may be faster to just clone a User, but I didn't test so ¯\_(ツ)_/¯
    pub user: Rc<User>,
}

impl Context {
    pub fn new(db_conn: Rc<PooledConnection>, user: Rc<User>) -> Self {
        Self { db_conn, user }
    }

    /// Get a reference to the DB connection.
    pub fn conn(&self) -> &PgConnection {
        &self.db_conn
    }
}

/// A mapping of the variable parts of a path. This maps each variable path
/// segment's identifier to its value in a particular path. For any particular
/// virtual node, the keys in this map are fixed, only the values vary.
///
/// NOTE: There will NOT be a variable defined for the last segment in the path,
/// i.e. the segment being operated on. This is because the variable would just
/// be a duplicate of the value that gets passed around anyway (see example).
///
/// Ideally, instead of a map we would have fixed identifiers, but that requires
/// macro wizardry that I just don't want to write. Example:
///
/// - Path definition: "/hardware/<hw_spec_slug>/programs/<program_spec_slug>"
/// - Path: "/hardware/hw1/programs/prog1"
/// - Variables:
///     - "hardware_spec_slug":"hw1"
#[derive(Clone, Debug)]
pub struct PathVariables {
    map: HashMap<String, String>,
}

impl PathVariables {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Helper for accessing path variables by name. All variable names are
    /// fixed at compile time, and all variables must be present in a path in
    /// order for it to match, so this should always succeed. A missing variable
    /// indicates a bug, hence why it panics in that case.
    pub fn get_var(&self, var_name: &'static str) -> &str {
        match self.map.get(var_name) {
            Some(value) => value,
            None => panic!(format!("Unknown path variable: {}", var_name)),
        }
    }

    /// Insert a new path segment into the map, if the given segment is
    /// variable. If the segment is fixed, this does nothing.
    pub fn insert_if_var(&mut self, path_segment: SegmentSpec, value: &str) {
        // If the segment is variable, store this value in the context
        if let SegmentSpec::Variable(var_name) = path_segment {
            self.map.insert(var_name.into(), value.into());
        }
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
    fn exists(
        &self,
        _context: &Context,
        _path_variables: &PathVariables,
        _path_segment: &str,
    ) -> Result<bool> {
        // This default implementation covers fixed virtual nodes, which will
        // generally exist as long as their parent exists. Override this if
        // you have a variable node, or if a fixed node can exist conditionally.
        Ok(true)
    }

    /// Gets the permission for the physical node at the given path segment.
    fn permissions(
        &self,
        context: &Context,
        path_variables: &PathVariables,
        path_segment: &str,
    ) -> Result<NodePermissions>;

    /// Gets the contents of the file at the given path segment.
    fn content(
        &self,
        _context: &Context,
        _path_variables: &PathVariables,
        _path_segment: &str,
    ) -> Result<String> {
        // This only needs to be implemented for files. For directories, this
        // should never be called because the NodeReference wrapper checks the
        // node type.
        panic!("Operation not supported for this virtual node")
    }

    /// Lists all physical nodes that exist for this variable virtual node.
    /// This should only be called for variable nodes. Fixed nodes
    /// will panic! Unlike the other operations on this trait, this function
    /// doesn't take a path segment, because it doesn't operate on a single
    /// physical node.
    fn list_variable_nodes(
        &self,
        _context: &Context,
        _path_variables: &PathVariables,
    ) -> Result<Vec<String>> {
        // This only needs to be implemented for directories with variable path
        // segments. For files and fixed directories, this should never be
        // called.
        panic!("Operation not supported for this virtual node")
    }

    /// Create a physical node at the given path segment. This will perform
    /// whatever operations are necessary so that a physical node will exist
    /// for this virtual node at that name.
    fn create_node(
        &self,
        _context: &Context,
        _path_variables: &PathVariables,
        _path_segment: &str,
    ) -> Result<()> {
        // Implement this for children of writable directories that can have
        // files created
        panic!("Operation not supported for this virtual node")
    }

    /// Set the content of the physical node at the given path segment.
    fn set_content(
        &self,
        _context: &Context,
        _path_variables: &PathVariables,
        _path_segment: &str,
        _content: &str,
    ) -> Result<()> {
        // This only needs to be implemented for writable files.
        panic!("Operation not supported for this virtual node")
    }

    /// Delete the physical node at the given path segment.
    fn delete(
        &self,
        _context: &Context,
        _path_variables: &PathVariables,
        _path_segment: &str,
    ) -> Result<()> {
        // This only needs to be implemented for writable files.
        panic!("Operation not supported for this virtual node")
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
    /// Construct a new virtual node that points to files.
    pub const fn file(
        path_segment: SegmentSpec,
        handler: &'static impl VirtualNodeHandler,
    ) -> Self {
        Self {
            path_segment,
            node_type: NodeType::File,
            handler,
            children: &[],
        }
    }

    /// Construct a new virtual node that points to directories.
    pub const fn dir(
        path_segment: SegmentSpec,
        handler: &'static impl VirtualNodeHandler,
        children: &'static [Self],
    ) -> Self {
        Self {
            path_segment,
            node_type: NodeType::Directory,
            handler,
            children,
        }
    }

    /// Finds the child of this virtual node of this node that matches the given
    /// path segment. The child will also be a virtual node.
    pub fn find_child(&self, path_segment: &str) -> Option<&'static Self> {
        self.children
            .iter()
            .find(|child| child.path_segment.matches(path_segment))
    }

    /// List the names of all physical nodes that exist for this virtual node.
    /// For fixed virtual nodes, this will always return either 0 or 1 names.
    /// For variable nodes it could return any number >= 0.
    pub fn list_physical_nodes(
        &self,
        context: &Context,
        path_variables: &PathVariables,
    ) -> Result<Vec<String>> {
        match self.path_segment {
            // For fixed nodes, we just need to check that the corresponding
            // physical node exists.
            SegmentSpec::Fixed(segment) => {
                Ok(if self.handler.exists(context, path_variables, segment)? {
                    vec![segment.into()]
                } else {
                    vec![]
                })
            }
            // For variable nodes, we'll need to dynamically
            // fetch a list of nodes.
            SegmentSpec::Variable(_) => {
                self.handler.list_variable_nodes(context, path_variables)
            }
        }
    }
}

/// Normalizes the given path, then splits it into segments. Normalization
/// includes de-duplicating slashes and removing trailing slashes.
///
/// In the future, if we want to start supporting relative paths, this could
/// resolve `.` and `..` as well.
///
/// ```
/// assert_eq!(split_path("/").as_slice(), &[""]);
/// assert_eq!(split_path("/dir1").as_slice(), &["", "dir1"]);
/// assert_eq!(split_path("/dir1/dir2/").as_slice(), &["", "dir1", "dir2"]);
/// ```
///
/// See unit tests for more test cases.
pub fn resolve_path(path: &str) -> Vec<&str> {
    lazy_static! {
        static ref PATH_SEP_RGX: Regex = Regex::new(r"/+").unwrap();
    }
    match path {
        // Special case, so that an empty path matches the root
        "" => vec![""],
        _ => PATH_SEP_RGX.split(path).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_path() {
        assert_eq!(resolve_path("").as_slice(), &[""]);
        assert_eq!(resolve_path("/").as_slice(), &[""]);
        assert_eq!(resolve_path("//").as_slice(), &[""]);
        assert_eq!(resolve_path("dir1").as_slice(), &["dir1"]);
        assert_eq!(resolve_path("/dir1").as_slice(), &["", "dir1"]);
        assert_eq!(
            resolve_path("/dir1/dir2").as_slice(),
            &["", "dir1", "dir2"]
        );
        assert_eq!(
            resolve_path("/dir1///dir2/").as_slice(),
            &["", "dir1", "dir2"]
        );
        assert_eq!(
            resolve_path("/dir1/dir2/file.txt").as_slice(),
            &["", "dir1", "dir2", "file.txt"]
        );
        assert_eq!(
            resolve_path("/dir1/./file.txt").as_slice(),
            &["", "dir1", ".", "file.txt"]
        );
        assert_eq!(
            resolve_path("/dir1/../file.txt").as_slice(),
            &["", "dir1", "..", "file.txt"]
        );
    }
}
