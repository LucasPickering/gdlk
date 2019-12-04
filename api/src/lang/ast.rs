/// The type of every value in our language
pub type LangValue = i32;

/// A symbol used to identify a certain stack
pub type StackIdentifier = usize;

// ===== AST types =====

/// An operator that takes no arguments. All operators that fit that description
/// should be grouped into this type.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NullaryOp {
    /// Reads one value from the input buffer to the workspace
    Read,
    /// Writes the workspace to the output buffer
    Write,
}

/// An operator whose arguments are (`LangValue`). All operators that fit
/// that description should be grouped into this type.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ValueOp {
    /// Sets the workspace to the given value
    Set,
    /// Adds a constant value to the workspace
    Add,
    /// Substracts a constant value from the workspace
    Sub,
    /// Multiplies the workspace by a constant value
    Mul,
}

/// An operator whose arguments are (`StackIdentifier`). All operators that fit
/// that description should be grouped into this type.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StackOp {
    /// Pushes the workspace onto the given stack
    Push,
    /// Pops the top value off the given stack into the workspace
    Pop,
}

#[derive(Debug, PartialEq)]
pub enum Instr {
    /// An operator takes no arguments
    NullaryOp(NullaryOp),
    /// An operator that consumes one `LangValue`
    ValueOp(ValueOp, LangValue),
    /// An operator that consumes one `StackIdentifier`
    StackOp(StackOp, StackIdentifier),

    // Control flow
    /// Executes the body if the workspace is != 0
    If(Vec<Instr>),
    /// Executes the body while the workspace is != 0
    While(Vec<Instr>),
}

/// A parsed program, i.e. an Abstract Syntax Tree.
#[derive(Debug, PartialEq)]
pub struct Program {
    pub body: Vec<Instr>,
}

/// An instruction set that is ready to be executed by a [Machine](Machine).
/// This instruction set is as minimal as possible, to reduce the complexity
/// of the runtime.
#[derive(Debug, PartialEq)]
pub enum MachineInstr {
    /// An operator takes no arguments
    NullaryOp(NullaryOp),
    /// An operator that consumes one `LangValue`
    ValueOp(ValueOp, LangValue),
    /// An operator that consumes one `StackIdentifier`
    StackOp(StackOp, StackIdentifier),

    // Jumps are relative: In `Jmp(n)`, `n` is relative to the current program
    // counter.
    // - `Jmp(-1)` repeats the previous instruction
    // - `Jmp(0)` repeats this instruction (so create an infinite loop)
    // - `Jmp(1)` goes to the next instruction (a no-op)
    // - `Jmp(2)` skips the next instruction
    // - etc...
    /// Jumps `n` relative instructions if the workspace == 0
    Jez(i32),
    /// Jumps `n` relative instructions if the workspace != 0
    Jnz(i32),
}
