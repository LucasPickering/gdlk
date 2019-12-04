/// The type of every value in our language
pub type LangValue = i32;

/// A name used to identify a certain register
pub type Register = usize;

/// A symbol used to identify a certain stack
pub type StackIdentifier = usize;

// ===== AST types =====

/// An operator is a special type of instruction that is guaranteed to be the
/// same in both ASTs. These are pulled into a separate subtype, so that they
/// can easily be shared between the two ASTs. This simplifies the AST
/// declarations as well as tree transformations.
///
/// An operator should never jump. This allows simplification of execution code,
/// because we know that each operator will immediately progress to the next
/// instruction.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Operator {
    /// Reads one value from the input buffer to a register
    Read(Register),
    /// Writes the value in a register to the output buffer
    Write(Register),
    /// Adds the value in the first register to the second register
    Add(Register, Register),
    /// Subtracts the value in the first register from the second register
    Sub(Register, Register),
    /// Multiplies the second register by the value in the first register
    Mul(Register, Register),
    /// Sets a register to a constant value
    Set(Register, LangValue),
    /// Pushes the value in a register onto the given stack
    Push(Register, StackIdentifier),
    /// Pops the top value off the given stack into a register
    Pop(StackIdentifier, Register),
}

#[derive(Debug, PartialEq)]
pub enum Instr {
    /// A simple operator (see [Operator](Operator))
    Operator(Operator),
    /// Executes the body if the register is != 0
    If(Register, Vec<Instr>),
    /// Executes the body while the register is != 0
    While(Register, Vec<Instr>),
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
    /// A simple operator (see [Operator](Operator))
    Operator(Operator),

    // Jumps are relative: In `Jmp(n)`, `n` is relative to the current program
    // counter.
    // - `Jmp(-1)` repeats the previous instruction
    // - `Jmp(0)` repeats this instruction (so create an infinite loop)
    // - `Jmp(1)` goes to the next instruction (a no-op)
    // - `Jmp(2)` skips the next instruction
    // - etc...
    /// Jumps `n` relative instructions if the register == 0
    Jez(i32, Register),
    /// Jumps `n` relative instructions if the register != 0
    Jnz(i32, Register),
}
