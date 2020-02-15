//! This module holds all the different types that can appear in our ASTs. There
//! is no functionality implemented here, just basic types.

use crate::consts::{REG_INPUT_LEN, REG_STACK_LEN_PREFIX, REG_USER_PREFIX};
use serde::Serialize;
use std::fmt::{self, Display, Formatter};

/// The type of every value in our language
pub type LangValue = i32;

/// A symbol used to identify a certain user register
pub type UserRegisterIdentifier = usize;

/// A symbol used to identify a certain stack
pub type StackIdentifier = usize;

/// A label for a certain point in the code
pub type Label = String;

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

/// Something that can produce a <LangValue> idempotently. The value
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
///
/// NOTE: All arithmetic operations are wrapping (for overflow/underflow).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Operator {
    /// Reads one value from the input buffer to a register
    Read(RegisterRef),
    /// Writes a value to the output buffer
    Write(ValueSource),
    /// Sets a register to a value
    Set(RegisterRef, ValueSource),
    /// Adds two values. Puts the result in the first argument.
    Add(RegisterRef, ValueSource),
    /// Subtracts the second value from the first. Puts the result in the
    /// first argument.
    Sub(RegisterRef, ValueSource),
    /// Multiplies the two values. Puts the result in the first argument.
    Mul(RegisterRef, ValueSource),
    /// Compares the last two arguments, and stores the comparison result in
    /// the first register. Result is -1 if the first value is less than the
    /// second, 0 if they are equal, and 1 if the first value is greater. The
    /// result will **never** be any value other than -1, 0, or 1.
    ///
    /// TODO: maybe we should remove this op?
    Cmp(RegisterRef, ValueSource, ValueSource),
    /// Pushes the value in a register onto the given stack
    Push(ValueSource, StackIdentifier),
    /// Pops the top value off the given stack into a register
    Pop(StackIdentifier, RegisterRef),
}

/// The different types of jumps. This just holds the jump type and conditional
/// value, not the jump target. That should be held by the parent, because the
/// target type can vary (label vs offset).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Jump {
    /// Jumps unconditionally
    Jmp,
    /// Jumps if the value == 0
    Jez(ValueSource),
    /// Jumps if the value != 0
    Jnz(ValueSource),
    /// Jumps if the value > 0
    Jlz(ValueSource),
    /// Jumps if the value < 0
    Jgz(ValueSource),
}

/// All types unique to the source AST live here
pub mod source {
    use super::*;

    /// A statement is one complete parseable element. Generally, each statement
    /// goes on its own line in the source.
    #[derive(Clone, Debug, PartialEq)]
    pub enum Statement {
        Label(Label),
        Operator(Operator),
        /// Jump to the given label
        Jump(Jump, Label),
    }

    /// A parsed and untransformed program
    #[derive(Clone, Debug, PartialEq)]
    pub struct Program {
        pub body: Vec<Statement>,
    }
}

/// All types unique to the compiled AST live here
pub mod compiled {
    use super::*;

    /// An executable instruction. These are the instructions that machines
    /// actually execute.
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum Instruction {
        Operator(Operator),
        /// These jumps are relative: In `Jmp(n)`, `n` is relative to the
        /// current program counter.
        /// - `Jmp(-1)` repeats the previous instruction
        /// - `Jmp(0)` repeats this instruction (so create an infinite loop)
        /// - `Jmp(1)` goes to the next instruction (a no-op)
        /// - `Jmp(2)` skips the next instruction
        /// - etc...
        Jump(Jump, isize),
    }

    /// A compiled program, ready to be executed
    #[derive(Clone, Debug, PartialEq)]
    pub struct Program {
        pub instructions: Vec<Instruction>,
    }
}
