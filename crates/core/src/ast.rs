//! All the different types that can appear in the different GDLK Abstract
//! Syntax Trees. There is no functionality implemented here, just basic types.
//! Every AST node type is generic and can hold an extra value. This is useful
//! to carry metadata along with the AST (e.g. [Span]).

use crate::{
    consts::{
        INPUT_LENGTH_REGISTER_REF, NULL_REGISTER_REF,
        STACK_LENGTH_REGISTER_REF_TAG, STACK_REF_TAG, USER_REGISTER_REF_TAG,
    },
    util::Span,
};
use std::fmt::{self, Display, Formatter};

/// The type of every value in our language.
pub type LangValue = i32;

/// A symbol used to identify a certain user register.
pub type UserRegisterId = usize;

/// A symbol used to identify a certain stack.
pub type StackId = usize;

/// A label for a certain point in the code.
pub type Label = String;

/// A generic AST node container. This holds the AST node data itself, as well
/// as some metadata (e.g. source span).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Node<T, M>(pub T, pub M);

impl<T, M> Node<T, M> {
    /// Get the data for this node.
    pub fn value(&self) -> &T {
        &self.0
    }

    /// Get the metadata for this node.
    pub fn metadata(&self) -> &M {
        &self.1
    }

    /// Create a new `Node` by mapping the data field using the given function.
    /// The metadata for the new node will remain the same.
    pub fn map<U>(self, mapper: impl Fn(T) -> U) -> Node<U, M> {
        Node(mapper(self.0), self.1)
    }
}

/// An alias for the node type that we use most commonly throughout the
/// compiler. Pairs each AST node with the original source that created it.
pub(crate) type SpanNode<T> = Node<T, Span>;

/// A reference to a stack, e.g. "S0". This should NOT be used for other uses
/// of a stack ID, e.g. in the register "RS0".
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct StackRef(pub StackId);

impl Display for StackRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", STACK_REF_TAG, self.0)
    }
}

/// A reference to a register. Registers can be readonly (in which case the
/// value is a reflection of some other part of state), or read-write, which
/// means the user can read and write freely from/to it.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum RegisterRef {
    /// This register is both readable and writable, but it also produces zero
    /// when read from, and anything written to it is thrown away.
    Null,
    /// Read-only register that provides the number of elements remaining
    /// in the input buffer
    InputLength,
    /// Read-only register that provides the current length of (i.e. the number
    /// of elements stored in) the referenced stack
    StackLength(StackId),
    /// User-writable register to be used for arbitrary computations
    User(UserRegisterId),
}

impl Display for RegisterRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => write!(f, "{}", NULL_REGISTER_REF),
            Self::InputLength => write!(f, "{}", INPUT_LENGTH_REGISTER_REF),
            Self::StackLength(stack_id) => {
                write!(f, "{}{}", STACK_LENGTH_REGISTER_REF_TAG, stack_id)
            }
            Self::User(reg_id) => {
                write!(f, "{}{}", USER_REGISTER_REF_TAG, reg_id)
            }
        }
    }
}

/// Something that can produce a [LangValue] idempotently. The value
/// can be read (repeatedly if necessary), but cannot be written to.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ValueSource<T> {
    /// A static value, fixed at build time
    Const(Node<LangValue, T>),
    /// A register, which can be read from to get a value
    Register(Node<RegisterRef, T>),
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
pub enum Operator<T> {
    /// Reads one value from the input buffer to a register. If the input is
    /// empty, triggers a runtime error.
    Read(Node<RegisterRef, T>),
    /// Writes a value to the output buffer.
    Write(Node<ValueSource<T>, T>),
    /// Sets a register to a value.
    Set(Node<RegisterRef, T>, Node<ValueSource<T>, T>),
    /// Adds two values. Puts the result in the first argument.
    Add(Node<RegisterRef, T>, Node<ValueSource<T>, T>),
    /// Subtracts the second value from the first. Puts the result in the
    /// first argument.
    Sub(Node<RegisterRef, T>, Node<ValueSource<T>, T>),
    /// Multiplies the two values. Puts the result in the first argument.
    Mul(Node<RegisterRef, T>, Node<ValueSource<T>, T>),
    /// Divides the first value by the second. Puts the result in the first
    /// argument. Any remainder from the division is thrown away, i.e. the
    /// result is floored. If the divisor is zero, triggers a runtime error.
    Div(Node<RegisterRef, T>, Node<ValueSource<T>, T>),
    /// Compares the last two arguments, and stores the comparison result in
    /// the first register. Result is -1 if the first value is less than the
    /// second, 0 if they are equal, and 1 if the first value is greater. The
    /// result will **never** be any value other than -1, 0, or 1.
    Cmp(
        Node<RegisterRef, T>,
        Node<ValueSource<T>, T>,
        Node<ValueSource<T>, T>,
    ),
    /// Pushes the value in a register onto the given stack. If the stack is
    /// already at capacity, triggers a runtime error.
    Push(Node<ValueSource<T>, T>, Node<StackRef, T>),
    /// Pops the top value off the given stack into a register. If the stack is
    /// empty, triggers a runtime error.
    Pop(Node<StackRef, T>, Node<RegisterRef, T>),
}

/// The different types of jumps. This just holds the jump type and conditional
/// value, not the jump target. That should be held by the parent, because the
/// target type can vary (label vs offset).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Jump<T> {
    /// Jumps unconditionally
    Jmp,
    /// Jumps if the value == 0
    Jez(Node<ValueSource<T>, T>),
    /// Jumps if the value != 0
    Jnz(Node<ValueSource<T>, T>),
    /// Jumps if the value > 0
    Jlz(Node<ValueSource<T>, T>),
    /// Jumps if the value < 0
    Jgz(Node<ValueSource<T>, T>),
}

/// All types unique to the source AST live here.
pub mod source {
    use super::*;

    /// A label declaration, e.g. "LABEL:".
    #[derive(Clone, Debug, PartialEq)]
    pub struct LabelDecl(pub Label);

    /// A statement is one complete parseable element. Generally, each statement
    /// goes on its own line in the source.
    #[derive(Clone, Debug, PartialEq)]
    pub enum Statement<T> {
        /// A label declaration
        Label(Node<LabelDecl, T>),
        /// See [Operator]
        Operator(Node<Operator<T>, T>),
        /// Jump to the given label
        Jump(Node<Jump<T>, T>, Node<Label, T>),
    }

    /// A parsed and untransformed program.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Program<T> {
        pub body: Vec<Node<Statement<T>, T>>,
    }
}

/// All types unique to the compiled AST live here.
pub mod compiled {
    use super::*;
    use crate::ProgramStats;

    /// An executable instruction. These are the instructions that machines
    /// actually execute.
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum Instruction<T> {
        /// See [Operator]
        Operator(Node<Operator<T>, T>),
        /// These jumps are relative: In `Jmp(n)`, `n` is relative to the
        /// current program counter.
        /// - `Jmp(-1)` repeats the previous instruction
        /// - `Jmp(0)` repeats this instruction (so create an infinite loop)
        /// - `Jmp(1)` goes to the next instruction (a no-op)
        /// - `Jmp(2)` skips the next instruction
        /// - etc...
        Jump(Node<Jump<T>, T>, isize),
    }

    /// A compiled program, ready to be executed.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Program<T> {
        pub instructions: Vec<Node<Instruction<T>, T>>,
        pub stats: ProgramStats,
    }

    impl<T> Program<T> {
        /// Get the number of compiled instructions in this program (does not
        /// include comments, whitespace, etc.).
        pub fn num_instructions(&self) -> usize {
            self.instructions.len()
        }

        /// Get the number of different USER registers (i.e. RXx) referenced by
        /// this program (not necessarily the number actually accessed).
        pub fn num_user_registers_referenced(&self) -> usize {
            self.stats
                .referenced_registers
                .iter()
                .filter(|reg_ref| matches!(reg_ref, RegisterRef::User(_)))
                .count()
        }

        /// Get the number of different stacks referenced by this program (not
        /// necessarily the number actually accessed).
        pub fn num_stacks_referenced(&self) -> usize {
            self.stats.referenced_stacks.len()
        }
    }
}

// Types that are only needed in wasm.
#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use crate::Span;
    use serde::{Deserialize, Serialize};
    use wasm_bindgen::prelude::*;

    /// Something that can map to source code. This can be some AST node, or
    /// an error, or something similar.
    #[wasm_bindgen]
    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
    pub struct SourceElement {
        #[wasm_bindgen(skip)]
        pub text: String,
        #[wasm_bindgen(readonly)]
        pub span: Span,
    }

    #[wasm_bindgen]
    impl SourceElement {
        #[wasm_bindgen(getter)]
        pub fn text(&self) -> String {
            self.text.clone()
        }
    }

    // Types that we can't natively return. These are assigned TS types, but
    // these types aren't actually verified by the compiler. Be careful
    // here!
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(typescript_type = "string[]")]
        pub type StringArray;

        #[wasm_bindgen(typescript_type = "Record<string, number>")]
        pub type LangValueMap;

        #[wasm_bindgen(typescript_type = "Record<string, number[]>")]
        pub type LangValueArrayMap;

        #[wasm_bindgen(typescript_type = "SourceElement[]")]
        pub type SourceElementArray;
    }
}
