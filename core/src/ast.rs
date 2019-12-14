//! This module holds all the different types that can appear in our ASTs. There
//! should be little to no functionality implemented here, just basic structs.

use crate::consts::{REG_INPUT_LEN, REG_STACK_LEN_PREFIX, REG_USER_PREFIX};
use serde::Serialize;
use std::fmt::{self, Display, Formatter};

/// The type of every value in our language
pub type LangValue = i32;

/// A symbol used to identify a certain user register
pub type UserRegisterIdentifier = usize;

/// A symbol used to identify a certain stack
pub type StackIdentifier = usize;

/// A reference to a register. Registers can be readonly (in which case the
/// value is a reflection of some other part of state), or read-write, which
/// means the user can read and write freely from/to it.
#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub enum RegisterRef {
    /// Read-only register that provides the number of elements remaining
    /// in the input buffer
    InputLength,
    /// Read-only register that provides the current length of (i.e. the number
    /// of elements stored in) the referenced stack
    StackLength(StackIdentifier),
    /// User-writable register to be used for arbitrary computations
    User(UserRegisterIdentifier),
}

// Need this impl so we can embed this type in error
impl Display for RegisterRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputLength => write!(f, "{}", REG_INPUT_LEN),
            Self::StackLength(stack_id) => {
                write!(f, "{}{}", REG_STACK_LEN_PREFIX, stack_id)
            }
            Self::User(reg_id) => write!(f, "{}{}", REG_USER_PREFIX, reg_id),
        }
    }
}

/// Something that can produce a [LangValue](LangValue) idempotently. The value
/// can be read (repeatedly if necessary), but cannot be written to.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ValueSource {
    /// A static value, fixed at build time
    Const(LangValue),
    /// A register, which can be read from to get a value
    Register(RegisterRef),
}

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
    Read(RegisterRef),
    /// Writes the value in a register to the output buffer
    Write(RegisterRef),
    /// Sets a register to a constant value
    Set(RegisterRef, ValueSource),
    /// Adds the two registers. Puts the result in the first register.
    Add(RegisterRef, ValueSource),
    /// Subtracts the second register from the first. Puts the result in the
    /// first register.
    Sub(RegisterRef, ValueSource),
    /// Multiplies the two registers. Puts the result in the first register.
    Mul(RegisterRef, ValueSource),
    /// Compares the last two arguments, and stores the comparison result in
    /// the first register. Result is -1 if the first value is less than the
    /// second, 0 if they are equal, and 1 if the first value is greater. The
    /// result will **never** be any value other than -1, 0, or 1.
    Cmp(RegisterRef, ValueSource, ValueSource),
    /// Pushes the value in a register onto the given stack
    Push(ValueSource, StackIdentifier),
    /// Pops the top value off the given stack into a register
    Pop(StackIdentifier, RegisterRef),
}

#[derive(Debug, PartialEq)]
pub enum Instr {
    /// A simple operator (see [Operator](Operator))
    Operator(Operator),
    /// Executes the body if the register is != 0
    If(RegisterRef, Vec<Instr>),
    /// Executes the body while the register is != 0
    While(RegisterRef, Vec<Instr>),
}

/// A parsed program, i.e. an Abstract Syntax Tree.
#[derive(Debug, PartialEq)]
pub struct Program {
    pub body: Vec<Instr>,
}

/// An instruction set that is ready to be executed by a
/// [Machine](crate::Machine). This instruction set is as minimal as possible,
/// to reduce the complexity of the runtime.
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
    Jez(i32, RegisterRef),
    /// Jumps `n` relative instructions if the register != 0
    Jnz(i32, RegisterRef),
}
