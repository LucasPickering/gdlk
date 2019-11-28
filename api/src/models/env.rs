use crate::lang::LangValue;
use serde::{Deserialize, Serialize};

/// The environment surrounding a program. This is needed both at compile time
/// and runtime. In the context of the game, this represents one puzzle.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Environment {
    /// Maximum number of stacks permitted
    pub num_stacks: usize,
    /// Maximum size of each stack. If None, the capacity is unlimited.
    pub max_stack_size: Option<usize>,
    /// The input values, where the element at position 0 is the first one that
    /// will be popped off.
    pub input: Vec<LangValue>,
    /// The correct value to be left in the output when the program exits. The
    /// first element will be the first one pushed, and so on.
    pub expected_output: Vec<LangValue>,
}
