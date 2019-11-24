use serde::Deserialize;

/// The type of every value in our language
pub type LangValue = i32;

/// A symbol used to identify a certain stack
pub type StackIdentifier = usize;

/// The environment surrounding a program. This is needed both at compile time
/// and runtime. In the context of the game, this represents one puzzle.
#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq, Deserialize)]
pub enum Instr {
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
    If(Vec<Instr>),
    /// Executes the body while the workspace is != 0
    While(Vec<Instr>),
}

/// A parsed program, i.e. an Abstract Syntax Tree.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Program {
    pub body: Vec<Instr>,
}

/// An instruction that has been desugared (hence "diet") and flattened such
/// that it has no nested bodies and its instruction set is as minimal as
/// possible. This is meant to be similar to asm, where there's no nested
/// instructions so that it's easy to track a program counter.
#[derive(Debug, PartialEq)]
pub enum DietInstr {
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
    /// A unique label, to be used as the target of jumps
    Label(String),
    /// Jumps to the specified label if the workspace == 0
    Jez(String),
    /// Jumps to the specified label if the workspace != 0
    Jnz(String),
}

/// An instruction set that is ready to be executed by a [Machine](Machine).
/// This instruction set is as minimal as possible, to reduce the complexity
/// of the runtime.
#[derive(Debug, PartialEq)]
pub enum MachineInstr {
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
    /// Jumps to the specified instruction index if the workspace == 0
    Jez(usize),
    /// Jumps to the specified instruction index if the workspace != 0
    Jnz(usize),
}
