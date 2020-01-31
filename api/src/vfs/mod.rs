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
    gql::GqlContext,
    models::User,
    util::PooledConnection,
    vfs::{
        hardware::{HardwareSpecFileNodeHandler, HardwareSpecNodeHandler},
        internal::{
            Context, PathVariables, SegmentSpec, VirtualNode,
            VirtualNodeHandler,
        },
        program::{
            ProgramSourceNodeHandler, ProgramSpecFileNodeHandler,
            ProgramSpecNodeHandler,
        },
    },
};
use juniper::{FieldResult, GraphQLEnum, GraphQLObject};
use std::rc::Rc;

#[derive(Copy, Clone, Debug, PartialEq, GraphQLEnum)]
pub enum NodeType {
    File,
    Directory,
}

#[derive(Copy, Clone, Debug, PartialEq, GraphQLObject)]
pub struct NodePermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

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

/// A physical node in the file system. This represents exactly one
/// file/directory. All file operations exist on this type.
///
/// To get a reference to a node, you can use <VirtualFileSystem::get_node>.
///
/// This type needs to be able to outlive the <VirtualFileSystem> that creates
/// it, so that it can be passed around for GraphQL purposes. Because of that,
/// it owns its own copy of <Context>, and that copy holds <Rc>s instead of
/// references.
#[derive(Clone)]
pub struct Node {
    context: Context,
    path_variables: PathVariables,
    path_segment: String,
    vnode: &'static VirtualNode,
}

// File operations that can be run on a Node
impl Node {
    pub fn name(&self) -> &str {
        &self.path_segment
    }

    pub fn node_type(&self) -> NodeType {
        self.vnode.node_type
    }

    pub fn permissions(&self) -> Result<NodePermissions> {
        self.vnode.handler.permissions(
            &self.context,
            &self.path_variables,
            &self.path_segment,
        )
    }

    /// Gets the actual data of this node. This is only possible for files. If
    /// called for a directory, will return
    /// <ServerError::UnsupportedFileOperation>.
    pub fn content(&self) -> Result<String> {
        match self.vnode.node_type {
            NodeType::File => self.vnode.handler.content(
                &self.context,
                &self.path_variables,
                &self.path_segment,
            ),
            // Can't do this for directories, ya dummy
            NodeType::Directory => Err(ServerError::UnsupportedFileOperation),
        }
    }

    /// Gets a list of each child of this node. This is only possible for
    /// directories, as files have no children. If called for a file, will
    /// return <ServerError::UnsupportedFileOperation>.
    pub fn children(&self) -> Result<Vec<Self>> {
        // Include this node's name as a variable for the children. Remember
        // that PathVariables doesn't ever contain the last path segment,
        // so the name of this node won't be in path_variables. We need to add
        // it now so that our children can access it. These clones are cheap
        // because the variables map is going to be small.
        let mut new_variables = self.path_variables.clone();
        new_variables
            .insert_if_var(self.vnode.path_segment, &self.path_segment);

        // Get each virtual child, paired with the name of each physical node
        // that exists for that virtual node.
        let child_vnodes: Vec<(&VirtualNode, Vec<String>)> =
            match self.node_type() {
                // Files don't have children!
                NodeType::File => {
                    return Err(ServerError::UnsupportedFileOperation)
                }
                NodeType::Directory => self
                    .vnode
                    .children
                    .iter()
                    // For each virtual child, collect its physical nodes
                    .map(|child_vnode| {
                        Ok((
                            child_vnode,
                            child_vnode.list_physical_nodes(
                                &self.context,
                                &new_variables,
                            )?,
                        ))
                    })
                    // Collect all results into one, and abort if any failed
                    .collect::<Result<Vec<(&VirtualNode, Vec<String>)>>>()?,
            };

        // Create a new Node for each child
        let mut child_nodes = Vec::new();
        for (child_vnode, child_names) in child_vnodes {
            child_nodes.extend(child_names.into_iter().map(|child_name| Self {
                context: self.context.clone(),
                path_variables: new_variables.clone(),
                path_segment: child_name,
                vnode: child_vnode,
            }))
        }

        Ok(child_nodes)
    }

    pub fn set_content(&self, content: String) -> Result<()> {
        match self.node_type() {
            NodeType::File => {
                // Always ask for permission!
                if self.permissions()?.write {
                    self.vnode.handler.set_content(
                        &self.context,
                        &self.path_variables,
                        &self.path_segment,
                        &content,
                    )
                } else {
                    Err(ServerError::PermissionDenied)
                }
            }
            NodeType::Directory => Err(ServerError::UnsupportedFileOperation),
        }
    }

    /// Delete this node. Any node with write permissions can be deleted. Any
    /// other node will generate a <ServerError::PermissionDenied>.
    pub fn delete(&self) -> Result<()> {
        if self.permissions()?.write {
            self.vnode.handler.delete(
                &self.context,
                &self.path_variables,
                &self.path_segment,
            )
        } else {
            Err(ServerError::PermissionDenied)
        }
    }
}

// GraphQL wrappers around the file operations
#[juniper::object(Context = GqlContext)]
impl Node {
    /// The name of this node, i.e. the last segment in the path that refers
    /// to this node.
    #[graphql(name = "name")]
    fn gql_name() -> String {
        self.name().into()
    }

    /// The type of this node (file or directory).
    #[graphql(name = "nodeType")]
    fn gql_node_type(&self) -> NodeType {
        self.node_type()
    }

    /// The permissions of this node (read/write/execute).
    #[graphql(name = "permissions")]
    fn gql_permissions() -> FieldResult<NodePermissions> {
        Ok(self.permissions()?)
    }

    /// The data of this node. Files have string content, while directories have
    /// no content. As such, this always returns content for files and `null`
    /// for directories.
    #[graphql(name = "content")]
    fn gql_content() -> FieldResult<Option<String>> {
        match self.content() {
            Ok(content) => Ok(Some(content)),
            Err(ServerError::UnsupportedFileOperation) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    /// The nested children of this node. Returns an array of the children for a
    /// directory, and `null` for a file.
    #[graphql(name = "children")]
    fn gql_children() -> FieldResult<Option<Vec<Self>>> {
        match self.children() {
            Ok(children) => Ok(Some(children)),
            Err(ServerError::UnsupportedFileOperation) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}

/// A container to hold wrappers for all GQL mutations that can be run on a
/// Node. These mutations all call down to methods that are defined on Node.
pub struct NodeMutation(Node);

impl NodeMutation {
    pub fn new(node: Node) -> Self {
        Self(node)
    }
}

#[juniper::object(Context = GqlContext)]
impl NodeMutation {
    /// Set the contents of a file. Will fail if the node is a directory or
    /// a file without write permissions.
    fn set_content(&self, content: String) -> FieldResult<&Node> {
        self.0.set_content(content)?;
        Ok(&self.0)
    }

    /// Delete a file or directory. Will fail if the node does not have write
    /// permissions. Returns the name (_not_ the full path) of the deleted file.
    fn delete(&self) -> FieldResult<String> {
        self.0.delete()?;
        Ok(self.0.name().into())
    }
}

/// A reference to the virtual file system. An instance of this struct allows
/// you to conduct operations on the VFS. All instances of this struct refer
/// to the same global VFS. This struct is constructed from whatever context
/// might be needed to serve file paths, e.g. DB connections.
///
/// This struct is useful for getting references to particular nodes (see
/// <Self::node>). Once you have a `Node`, you can run
/// file operations on it.
pub struct VirtualFileSystem {
    db_conn: Rc<PooledConnection>,
    user: Rc<User>,
}

impl VirtualFileSystem {
    /// Construct a new reference to the virtual file system. Construction is
    /// cheap, so it's fine to construct this only when it's needed, rather
    /// than maintaining a long-living instance.
    pub fn new(db_conn: Rc<PooledConnection>, user: Rc<User>) -> Self {
        VirtualFileSystem { db_conn, user }
    }

    /// Gets a reference to a particular file system node. This is a _physical_
    /// node, meaning it refers to exactly one node in the file system. This
    /// reference can be used to run operations on the node.
    pub fn get_node(&self, path: &str) -> Result<Node> {
        /// Checks if the first segment in the path matches the given virtual
        /// node. If so, returns a tuple of (matched segment, remaining
        /// unmatched segments). If it doesn't match, returns a `NodeNotFound`.
        fn match_first_segment<'a>(
            node: &VirtualNode,
            context: &Context,
            path_variables: &PathVariables,
            path_segments: &'a [&'a str],
        ) -> Result<(&'a str, &'a [&'a str])> {
            match path_segments {
                [first, rest @ ..]
                    if node.path_segment.matches(first)
                        && node.handler.exists(
                            context,
                            path_variables,
                            first,
                        )? =>
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
        fn find_node(
            node: &'static VirtualNode,
            context: &Context,
            path_variables: &mut PathVariables,
            path_segments: &[&str],
        ) -> Result<&'static VirtualNode> {
            // If the node doesn't match the first segment, we'll bail here
            let (first, rest) = match_first_segment(
                node,
                context,
                path_variables,
                path_segments,
            )?;
            match rest {
                // This is the last node, return it
                [] => Ok(node),
                // There is at least one more node, match it to this node's
                // children
                [second, ..] => match node.find_child(second) {
                    Some(child) => {
                        // We're adding in the _parent_ path segment here. We
                        // don't want to add the child path segment until it's
                        // not the final node (see Context definition).
                        path_variables.insert_if_var(node.path_segment, first);
                        find_node(child, context, path_variables, rest)
                    }
                    None => Err(ServerError::NodeNotFound),
                },
            }
        }

        let segments: Vec<&str> = internal::resolve_path(path);
        let context = Context::new(self.db_conn.clone(), self.user.clone());
        let mut path_variables = PathVariables::new();
        // Kick off the recursive process to walk down the tree
        let vnode = find_node(
            &VFS_TREE,
            &context,
            &mut path_variables,
            segments.as_slice(),
        )?;
        Ok(Node {
            context,
            path_variables,
            path_segment: (*segments.last().unwrap()).to_owned(),
            vnode,
        })
    }
}

/// Handler for any simple directory that doesn't need custom logic. Most
/// directories with fixed paths should be able to use this. Right now this is
/// hard-coded to RX permissions, but we could have it take a permission at
/// construction and return that from the method, to make it configurable.
#[derive(Debug)]
struct SimpleDirHandler();

impl VirtualNodeHandler for SimpleDirHandler {
    fn permissions(
        &self,
        _: &Context,
        _: &PathVariables,
        _: &str,
    ) -> Result<NodePermissions> {
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
                                SegmentSpec::Variable("program_source_name"),
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
        assert_err,
        models::{
            NewHardwareSpec, NewProgramSpec, NewUser, NewUserProgram, User,
        },
        schema::{hardware_specs, program_specs},
        util,
    };
    use diesel::RunQueryDsl;

    fn setup() -> VirtualFileSystem {
        let conn = util::test_db_conn();

        // Insert user
        NewUser { username: "user1" }
            .insert()
            .execute(&conn)
            .unwrap();
        let user: User =
            User::filter_by_username("user1").get_result(&conn).unwrap();

        // Hardware spec
        let hw_spec_id = NewHardwareSpec {
            slug: "hw1",
            num_registers: 1,
            num_stacks: 0,
            max_stack_length: 0,
        }
        .insert()
        .returning(hardware_specs::id)
        .get_result(&conn)
        .unwrap();

        // Program specs
        let prog1_spec_id = NewProgramSpec {
            slug: "prog1",
            hardware_spec_id: hw_spec_id,
            input: vec![],
            expected_output: vec![],
        }
        .insert()
        .returning(program_specs::id)
        .get_result(&conn)
        .unwrap();
        NewProgramSpec {
            slug: "prog2",
            hardware_spec_id: hw_spec_id,
            input: vec![],
            expected_output: vec![],
        }
        .insert()
        .execute(&conn)
        .unwrap();

        // Source code for one program
        NewUserProgram {
            user_id: user.id,
            program_spec_id: prog1_spec_id,
            file_name: "program.gdlk",
            source_code: "READ\nWRITE\n",
        }
        .insert()
        .execute(&conn)
        .unwrap();

        VirtualFileSystem::new(Rc::new(conn), Rc::new(user))
    }

    fn check_node_names(nodes: &[Node], expected_names: &[&str]) {
        assert_eq!(
            nodes.iter().map(Node::name).collect::<Vec<&str>>(),
            expected_names.iter().copied().collect::<Vec<&str>>()
        );
    }

    #[test]
    fn test_get_node() {
        let vfs = setup();
        // === negative test cases ===
        assert!(vfs.get_node("/fake/path/does/not/exist").is_err());
        assert!(vfs.get_node("hardware").is_err()); // relative paths don't work

        // Unknown hw and program IDs don't exist
        assert!(vfs.get_node("/hardware/hw2").is_err());
        assert!(vfs.get_node("/hardware/hw1/programs/prog100").is_err());
        assert!(vfs
            .get_node("/hardware/hw1/programs/prog1/program2.gdlk")
            .is_err());

        // === positive test cases ===
        assert!(vfs.get_node("").is_ok());
        assert!(vfs.get_node("/").is_ok());
        assert!(vfs.get_node("//").is_ok()); // slash gets de-duped
        assert!(vfs.get_node("/hardware").is_ok());
        assert!(vfs.get_node("/hardware/").is_ok());
        assert!(vfs.get_node("/hardware/hw1/spec.txt").is_ok());
        assert!(vfs.get_node("/hardware/hw1/programs/prog1").is_ok());
        assert!(vfs
            .get_node("/hardware/hw1/programs/prog1/spec.txt")
            .is_ok());
    }

    #[test]
    fn test_root_dir() {
        let vfs = setup();
        let node = vfs.get_node("/").unwrap();
        assert_eq!(node.name(), "");
        assert_eq!(node.node_type(), NodeType::Directory);
        assert_eq!(node.permissions().unwrap(), PERMS_RX);
        assert_err!(node.content(), "Operation not supported");
        assert_err!(node.set_content("test".into()), "Operation not supported");
        check_node_names(&node.children().unwrap(), &["hardware"]);
        assert_err!(node.delete(), "Permission denied");
    }

    #[test]
    fn test_hw_dir() {
        let vfs = setup();
        let node = vfs.get_node("/hardware").unwrap();
        assert_eq!(node.name(), "hardware");
        assert_eq!(node.node_type(), NodeType::Directory);
        assert_eq!(node.permissions().unwrap(), PERMS_RX);
        assert_err!(node.content(), "Operation not supported");
        check_node_names(&node.children().unwrap(), &["hw1"]);
        assert_err!(node.delete(), "Permission denied");
    }

    #[test]
    fn test_hw_spec() {
        let vfs = setup();

        let hw_spec_dir = vfs.get_node("/hardware/hw1").unwrap();
        assert_eq!(hw_spec_dir.name(), "hw1");
        assert_eq!(hw_spec_dir.node_type(), NodeType::Directory);
        assert_eq!(hw_spec_dir.permissions().unwrap(), PERMS_RX);
        assert_err!(hw_spec_dir.content(), "Operation not supported");
        check_node_names(
            &hw_spec_dir.children().unwrap(),
            &["spec.txt", "programs"],
        );
        assert_err!(hw_spec_dir.delete(), "Permission denied");

        let spec_file = vfs.get_node("/hardware/hw1/spec.txt").unwrap();
        assert_eq!(spec_file.name(), "spec.txt");
        assert_eq!(spec_file.node_type(), NodeType::File);
        assert_eq!(spec_file.permissions().unwrap(), PERMS_R);
        assert_eq!(
            spec_file.content().unwrap(),
            "Registers: 1\nStacks: 0\nMax stack length: 0\n"
        );
        assert_err!(spec_file.children(), "Operation not supported");
        assert_err!(spec_file.delete(), "Permission denied");
    }

    #[test]
    fn test_hw_programs_dir() {
        let vfs = setup();

        let node = vfs.get_node("/hardware/hw1/programs").unwrap();
        assert_eq!(node.name(), "programs");
        assert_eq!(node.node_type(), NodeType::Directory);
        assert_eq!(node.permissions().unwrap(), PERMS_RX);
        assert_err!(node.content(), "Operation not supported");
        check_node_names(&node.children().unwrap(), &["prog1", "prog2"]);
        assert_err!(node.delete(), "Permission denied");
    }

    #[test]
    fn test_program_spec_dir() {
        let vfs = setup();

        let program_spec_dir =
            vfs.get_node("/hardware/hw1/programs/prog1").unwrap();
        assert_eq!(program_spec_dir.name(), "prog1");
        assert_eq!(program_spec_dir.node_type(), NodeType::Directory);
        assert_eq!(program_spec_dir.permissions().unwrap(), PERMS_RX);
        assert_err!(program_spec_dir.content(), "Operation not supported");
        check_node_names(
            &program_spec_dir.children().unwrap(),
            &["spec.txt", "program.gdlk"],
        );
        assert_err!(program_spec_dir.delete(), "Permission denied");

        let spec_file = vfs
            .get_node("/hardware/hw1/programs/prog1/spec.txt")
            .unwrap();
        assert_eq!(spec_file.name(), "spec.txt");
        assert_eq!(spec_file.node_type(), NodeType::File);
        assert_eq!(spec_file.permissions().unwrap(), PERMS_R);
        assert_eq!(
            spec_file.content().unwrap(),
            "Input: []\nExpected output: []\n"
        );
        assert_err!(spec_file.children(), "Operation not supported");
        assert_err!(spec_file.delete(), "Permission denied");

        let source_file = vfs
            .get_node("/hardware/hw1/programs/prog1/program.gdlk")
            .unwrap();
        assert_eq!(source_file.name(), "program.gdlk");
        assert_eq!(source_file.node_type(), NodeType::File);
        assert_eq!(source_file.permissions().unwrap(), PERMS_RW);
        assert_eq!(source_file.content().unwrap(), "READ\nWRITE\n");
        assert_err!(source_file.children(), "Operation not supported");
        // Delete the file, and make sure it's gone
        assert!(source_file.delete().is_ok());
        assert_err!(
            vfs.get_node("/hardware/hw1/programs/prog1/program.gdlk"),
            "File or directory not found",
        );
    }

    #[test]
    fn test_nested_children() {
        let vfs = setup();

        // We want to test two different forms of getting nested child data:
        // - the last node in the path is fixed
        // - the last node in the path is variable
        // This will check for a possible bug with variable insertion, where
        // a leaf node's name won't get inserted into the variables before
        // fetching nested children.

        // Last node is fixed
        {
            let hw_dir = vfs.get_node("/hardware").unwrap();
            let hw_children = hw_dir.children().unwrap();
            let hw1_dir = hw_children.get(0).unwrap();
            assert_eq!(hw1_dir.name(), "hw1");
            let hw1_children = hw1_dir.children().unwrap();
            let hw1_spec_file = hw1_children.get(0).unwrap();
            assert_eq!(hw1_spec_file.name(), "spec.txt");
            assert_eq!(
                hw1_spec_file.content().unwrap(),
                "Registers: 1\nStacks: 0\nMax stack length: 0\n"
            );
        }

        // Last node is variable
        {
            let hw1_dir = vfs.get_node("/hardware/hw1").unwrap();
            assert_eq!(hw1_dir.name(), "hw1");
            let hw1_children = hw1_dir.children().unwrap();
            let hw1_spec_file = hw1_children.get(0).unwrap();
            assert_eq!(hw1_spec_file.name(), "spec.txt");
            assert_eq!(
                hw1_spec_file.content().unwrap(),
                "Registers: 1\nStacks: 0\nMax stack length: 0\n"
            );
        }
    }
}
