use crate::{lang::LangValue, schema::environments};
use diesel::{Identifiable, Queryable};
use serde::{Deserialize, Serialize};

/// The environment surrounding a program. This is needed both at compile time
/// and runtime. In the context of the game, this represents one puzzle.
#[derive(Debug, PartialEq, Serialize, Deserialize, Identifiable, Queryable)]
pub struct Environment {
    /// Unique ID to refer to this environment
    #[serde(default)]
    pub id: i32,

    // These three need to be i32s because postgres has no unsigned type.
    // The insertion code and DB should both enforce that they are >= 0.
    /// Number of registers available
    pub num_registers: i32,
    /// Maximum number of stacks permitted
    pub num_stacks: i32,
    /// Maximum size of each stack. If None, the capacity is unlimited.
    pub max_stack_size: Option<i32>,

    /// The input values, where the element at position 0 is the first one that
    /// will be popped off.
    pub input: Vec<LangValue>,
    /// The correct value to be left in the output when the program exits. The
    /// first element will be the first one pushed, and so on.
    pub expected_output: Vec<LangValue>,
}
