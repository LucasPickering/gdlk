//! A virtual file system. Used to serve objects that appear to be files and
//! directories, but in fact are backed by multiple data sources, including
//! static files on disk and information in the database. Some terms:
//!
//! - Path: A concrete pointer to a node, e.g. "/hardware/hw1"
//! - Path segment: One chunk of a path, i.e. the data between two slashes. A
//!   segment can be fixed or variable. See [SegmentSpec](SegmentSpec).
//! - File: A single file that has string data and metadata
//! - Directory: A collection of nodes, which has metadata but no string data of
//!   its own
//! - Node: A file or directory
//! - Virtual node: A Rust object that dynamically serves nodes at a particular
//!   path. The path can have dynamic components, e.g. /hardware/<slug>, so this
//!   can serve more than one concrete path.
//! - Node handler: A Rust object that defines how a particular virtual node
//!   serves its data. This is where the dynamic logic is defined, that allows
//!   the file system to compose data from different sources.

mod hardware;
mod internal;
mod program;

use crate::{
    error::{Result, ServerError},
    util,
    vfs::{
        hardware::{HardwareSpecFileNodeHandler, HardwareSpecNodeHandler},
        internal::{Context, SegmentSpec, VirtualNode, VirtualNodeHandler},
        program::ProgramSourceNodeHandler,
    },
};
use diesel::PgConnection;
use program::{ProgramSpecFileNodeHandler, ProgramSpecNodeHandler};
use serde::Serialize;
use std::{collections::HashMap, fmt::Debug};

const PERMS_R: NodePermissions = NodePermissions {
    read: true,
    write: false,
    execute: false,
};
const PERMS_RW: NodePermissions = NodePermissions {
    read: true,
    write: true,
    execute: false,
};
const PERMS_RX: NodePermissions = NodePermissions {
    read: true,
    write: false,
    execute: true,
};

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub enum NodeType {
    File,
    Directory,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct NodePermissions {
    read: bool,
    write: bool,
    execute: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NodeMetadata {
    node_type: NodeType,
    name: String,
    permissions: NodePermissions,
}

/// A reference to the virtual file system. An instance of this struct allows
/// you to conduct operations on the VFS. All instances of this struct refer
/// to the same global VFS. This struct is constructed from whatever context
/// might be needed to serve file paths, e.g. DB connections.
///
/// All external file operations are defined on this struct. Each operation
/// will require a full file path, and possibly more data, depending upon the
/// operation.
pub struct VirtualFileSystem<'a> {
    db_conn: &'a PgConnection,
}

impl<'a> VirtualFileSystem<'a> {
    /// Construct a new reference to the virtual file system.Construction is
    /// cheap, so it's fine to construct this only when it's needed, rather
    /// than maintaining a long-living instance.
    pub fn new(db_conn: &'a PgConnection) -> Self {
        Self { db_conn }
    }

    /// Helper to apply an operation to a path. This splits the path into
    /// segments, then follows the segments down the tree. At each step, it
    /// ensures the segment refers to a valid node before moving onto the next
    /// one. Once the last segment is reached, the given operation is applied
    /// to the corresponding node, and the output of that operation is returned.
    fn walk_tree<T>(
        &self,
        func: impl NodeOperation<T>,
        path: &str,
    ) -> Result<T> {
        /// Checks if the first segment in the path matches the given virtual
        /// node. If so, returns a tuple of (matched segment, remaining
        /// unmatched segments). If it doesn't match, returns a `NodeNotFound`.
        fn match_first_segment<'a>(
            node: &VirtualNode,
            context: &Context<'a>,
            path_segments: &'a [&'a str],
        ) -> Result<(&'a str, &'a [&'a str])> {
            match path_segments {
                [first, rest @ ..]
                    if node.path_segment.matches(first)
                        && node.exists(&context, first)? =>
                {
                    Ok((first, rest))
                }

                // path_segments is empty, the first segment doesn't match any
                // virtual node, or the corresponding physical node doesn't
                // exist
                _ => Err(ServerError::NodeNotFound),
            }
        }

        /// Recursive function that walks down each segment in the path,
        /// matching it to the appropriate next node. Once the end of the path
        /// is reached, the given operation is applied to the final node. If
        /// at any point there is no matching node, this will bail with a
        /// `NodeNotFound`. Each iteration of this function steps down one node
        /// in the tree.
        fn process_node<'a, T>(
            node: &VirtualNode,
            context: &mut Context<'a>,
            func: impl NodeOperation<T>,
            path_segments: &'a [&'a str],
        ) -> Result<T> {
            // If the node doesn't match the first segment, we'll bail here
            let (first, rest) =
                match_first_segment(node, context, path_segments)?;
            match rest {
                // This is the last node, run the operation on it
                [] => func(node, context, first),
                // There is at least one more node, match it to this node's
                // children
                [second, ..] => match node.match_child(second) {
                    Some(child) => {
                        // We're adding in the _parent_ path segment here. We
                        // don't want to add the child path segment until it's
                        // not the final node (see Context definition).
                        match node.path_segment {
                            // If the segment is variable, store this value
                            // in the context
                            SegmentSpec::Variable(var_name) => {
                                context.variables.insert(var_name, first);
                            }
                            SegmentSpec::Fixed(_) => {}
                        }
                        process_node(child, context, func, rest)
                    }
                    None => Err(ServerError::NodeNotFound),
                },
            }
        }

        let segments: Vec<&str> = util::resolve_path(path);
        let mut context = Context {
            db_conn: self.db_conn,
            variables: HashMap::new(),
        };
        // Kick off the recursive process to walk down the tree
        process_node(&VFS_TREE, &mut context, func, segments.as_slice())
    }

    /// Checks if the given path exists in the file system. Returns true if a
    /// physical node exists at that path, false if any node along the path does
    /// not exist, error if anything goes write during the operation.
    pub fn exists(&self, path: &str) -> Result<bool> {
        match self.walk_tree(VirtualNode::exists, path) {
            // It doesn't make any sense to return NodeNotFound from this
            Err(ServerError::NodeNotFound) => Ok(false),
            res => res,
        }
    }

    /// Gets the metadata for the physical node at a path. Returns an error if
    /// it does not exist or anything else goes wrong during the operation.
    pub fn get_metadata(&self, path: &str) -> Result<NodeMetadata> {
        self.walk_tree(VirtualNode::get_metadata, path)
    }

    /// Gets the contents of the file at the given path. Returns an error if
    /// the file does not exist, if the node at that path is not a file, or
    /// if anything else goes wrong during the operation.
    pub fn get_content(&self, path: &str) -> Result<String> {
        self.walk_tree(VirtualNode::get_content, path)
    }

    /// Lists all children of the directory at the given path. Returns an error
    /// if the directory does not exist, if the node at that path is not a
    /// directory, or if anything else goes wrong during the operation.
    pub fn list_children(&self, path: &str) -> Result<Vec<NodeMetadata>> {
        self.walk_tree(VirtualNode::list_children, path)
    }
}

/// A function that can be applied to a <VirtualNode>, returning
/// a result. Examples of operations are <VirtualNode::exists> and
/// <VirtualNode::get_metadata>. This is used to make node operations generic
/// so that they can use shared tree-walking code.
trait NodeOperation<T> = FnOnce(&VirtualNode, &Context, &str) -> Result<T>;

/// Handler for any simple directory that doesn't need custom logic. Most
/// directories with fixed paths should be able to use this. Right now this is
/// hard-coded to RX permissions, but we could have it take a permission at
/// construction and return that from the method, to make it configurable.
#[derive(Debug)]
struct SimpleDirHandler();

impl VirtualNodeHandler for SimpleDirHandler {
    fn get_permissions(&self, _: &Context, _: &str) -> Result<NodePermissions> {
        Ok(PERMS_RX)
    }
}

/// The entire VFS node tree. This defines the layout of the tree. Each node has
/// a path spec, a handler that defines how it behaves, and optionally children.
/// Obviously, only directions can have children.
static VFS_TREE: VirtualNode = VirtualNode::dir(
    // If you update something here, make sure to update the comment above!
    SegmentSpec::Fixed(""),
    &SimpleDirHandler(),
    &[VirtualNode::dir(
        SegmentSpec::Fixed("hardware"),
        &SimpleDirHandler(),
        &[VirtualNode::dir(
            SegmentSpec::Variable("hw_spec_slug"),
            &HardwareSpecNodeHandler(),
            &[
                VirtualNode::file(
                    SegmentSpec::Fixed("spec.txt"),
                    &HardwareSpecFileNodeHandler(),
                ),
                VirtualNode::dir(
                    SegmentSpec::Fixed("programs"),
                    &SimpleDirHandler(),
                    &[VirtualNode::dir(
                        SegmentSpec::Variable("program_spec_slug"),
                        &ProgramSpecNodeHandler(),
                        &[
                            VirtualNode::file(
                                SegmentSpec::Fixed("spec.txt"),
                                &ProgramSpecFileNodeHandler(),
                            ),
                            VirtualNode::file(
                                SegmentSpec::Fixed("program.gdlk"),
                                &ProgramSourceNodeHandler(),
                            ),
                        ],
                    )],
                ),
            ],
        )],
    )],
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        diesel::RunQueryDsl,
        models::{NewHardwareSpec, NewProgramSpec},
        schema::{hardware_specs, program_specs},
    };

    // Helper methods to make test data creation a bit cleaner
    impl NodeMetadata {
        fn file(name: &str, permissions: NodePermissions) -> Self {
            Self {
                node_type: NodeType::File,
                name: name.into(),
                permissions,
            }
        }

        fn dir(name: &str, permissions: NodePermissions) -> Self {
            Self {
                node_type: NodeType::Directory,
                name: name.into(),
                permissions,
            }
        }
    }

    fn insert_specs(
        conn: &PgConnection,
        hw_spec_slug: &str,
        program_spec_slug: &str,
    ) {
        let hw_spec_id = diesel::insert_into(hardware_specs::table)
            .values(&NewHardwareSpec {
                slug: hw_spec_slug.into(),
                num_registers: 1,
                num_stacks: 0,
                max_stack_length: 0,
            })
            .returning(hardware_specs::id)
            .get_result(conn)
            .unwrap();
        diesel::insert_into(program_specs::table)
            .values(&NewProgramSpec {
                slug: program_spec_slug.into(),
                hardware_spec_id: hw_spec_id,
                input: vec![],
                expected_output: vec![],
            })
            .execute(conn)
            .unwrap();
    }

    #[test]
    fn test_exists() {
        let conn = util::test_connection();
        let vfs = VirtualFileSystem::new(&conn);
        insert_specs(&conn, "hw1", "prog1");

        // negative test cases
        assert_eq!(vfs.exists("/fake/path/does/not/exist").unwrap(), false);
        // relative paths don't work
        assert_eq!(vfs.exists("hardware").unwrap(), false);
        assert_eq!(vfs.exists("/hardware/hw2").unwrap(), false);
        assert_eq!(vfs.exists("/hardware/hw1/programs/prog2").unwrap(), false);

        // positive test cases
        assert_eq!(vfs.exists("").unwrap(), true);
        assert_eq!(vfs.exists("/").unwrap(), true);
        assert_eq!(vfs.exists("//").unwrap(), true); // slash gets de-duped
        assert_eq!(vfs.exists("/hardware").unwrap(), true);
        assert_eq!(vfs.exists("/hardware/").unwrap(), true);
        assert_eq!(vfs.exists("/hardware/hw1/spec.txt").unwrap(), true);
        assert_eq!(vfs.exists("/hardware/hw1/programs/prog1").unwrap(), true);
        assert_eq!(
            vfs.exists("/hardware/hw1/programs/prog1/spec.txt").unwrap(),
            true
        );
    }

    #[test]
    fn test_metadata() {
        let conn = util::test_connection();
        let vfs = VirtualFileSystem::new(&conn);
        insert_specs(&conn, "hw1", "prog1");

        // success
        assert_eq!(
            vfs.get_metadata("/").unwrap(),
            NodeMetadata::dir("", PERMS_RX)
        );
        assert_eq!(
            vfs.get_metadata("/hardware").unwrap(),
            NodeMetadata::dir("hardware", PERMS_RX)
        );
        assert_eq!(
            vfs.get_metadata("/hardware/hw1").unwrap(),
            NodeMetadata::dir("hw1", PERMS_RX)
        );
        assert_eq!(
            vfs.get_metadata("/hardware/hw1/spec.txt").unwrap(),
            NodeMetadata::file("spec.txt", PERMS_R)
        );
        assert_eq!(
            vfs.get_metadata("/hardware/hw1/programs").unwrap(),
            NodeMetadata::dir("programs", PERMS_RX)
        );
        assert_eq!(
            vfs.get_metadata("/hardware/hw1/programs/prog1").unwrap(),
            NodeMetadata::dir("prog1", PERMS_RX)
        );
        assert_eq!(
            vfs.get_metadata("/hardware/hw1/programs/prog1/spec.txt")
                .unwrap(),
            NodeMetadata::file("spec.txt", PERMS_R)
        );
        assert_eq!(
            vfs.get_metadata("/hardware/hw1/programs/prog1/program.gdlk")
                .unwrap(),
            NodeMetadata::file("program.gdlk", PERMS_RW)
        );

        // errors
        assert_eq!(
            format!("{}", vfs.get_metadata("/fake").unwrap_err()),
            "File or directory not found"
        );
    }

    #[test]
    fn test_file_content() {
        let conn = util::test_connection();
        let vfs = VirtualFileSystem::new(&conn);
        insert_specs(&conn, "hw1", "prog1");

        // success
        assert_eq!(
            vfs.get_content("/hardware/hw1/spec.txt").unwrap(),
            String::from("Registers: 1\nStacks: 0\nMax stack length: 0\n")
        );
        assert_eq!(
            vfs.get_content("/hardware/hw1/programs/prog1/spec.txt")
                .unwrap(),
            String::from("Input: []\nExpected output: []\n")
        );
        assert_eq!(
            vfs.get_content("/hardware/hw1/programs/prog1/program.gdlk")
                .unwrap(),
            String::from("TODO")
        );

        // errors
        assert_eq!(
            format!("{}", vfs.get_content("/").unwrap_err()),
            "Operation not supported"
        );
        assert_eq!(
            format!("{}", vfs.get_content("/fake").unwrap_err()),
            "File or directory not found"
        );
    }

    #[test]
    fn test_list_children() {
        let conn = util::test_connection();
        let vfs = VirtualFileSystem::new(&conn);
        insert_specs(&conn, "hw1", "prog1");

        // success
        assert_eq!(
            vfs.list_children("/").unwrap(),
            vec![NodeMetadata::dir("hardware", PERMS_RX)]
        );
        assert_eq!(
            vfs.list_children("/hardware").unwrap(),
            vec![NodeMetadata::dir("hw1", PERMS_RX)]
        );
        assert_eq!(
            vfs.list_children("/hardware/hw1").unwrap(),
            vec![
                NodeMetadata::file("spec.txt", PERMS_R),
                NodeMetadata::dir("programs", PERMS_RX)
            ]
        );
        assert_eq!(
            vfs.list_children("/hardware/hw1/programs").unwrap(),
            vec![NodeMetadata::dir("prog1", PERMS_RX)]
        );
        assert_eq!(
            vfs.list_children("/hardware/hw1/programs/prog1").unwrap(),
            vec![
                NodeMetadata::file("spec.txt", PERMS_R),
                NodeMetadata::file("program.gdlk", PERMS_RW)
            ],
        );

        // errors
        assert_eq!(
            format!(
                "{}",
                vfs.list_children("/hardware/hw1/spec.txt").unwrap_err()
            ),
            "Operation not supported"
        );
        assert_eq!(
            format!("{}", vfs.list_children("/fake").unwrap_err()),
            "File or directory not found"
        );
    }
}
