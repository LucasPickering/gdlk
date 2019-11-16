use serde::{Deserialize, Serialize};

/// The type of every value in our language
pub type LangValue = i32;

/// A symbol used to identify a certain stack
pub type StackIdentifier = usize;

/// The environment surrounding a program. This is needed both at compile time
/// and runtime. In the context of the game, this represents one puzzle.
#[derive(Debug)]
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

// ===== AST types =====

#[derive(Debug, Serialize, Deserialize)]
pub enum Instruction {
    /// Reads one value from the input buffer to the workspace
    Read,
    /// Writes the workspace to the output buffer
    Write,
    /// Sets the workspace to the given value
    Set(LangValue),
    /// Pushes the workspace onto the given stack
    Push(StackIdentifier),
    /// Pops the top value off the given stack into the workspace
    Pop(StackIdentifier),
    /// Executes the body if the workspace is != 0
    If(Vec<Instruction>),
    /// Executes the body while the workspace is != 0
    While(Vec<Instruction>),
}

/// A parsed program, i.e. an Abstract Syntax Tree.
#[derive(Debug, Serialize, Deserialize)]
pub struct Program {
    pub body: Vec<Instruction>,
}
