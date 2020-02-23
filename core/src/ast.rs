//! All the different types that can appear in the different GDLK Abstract
//! Syntax Trees. There is no functionality implemented here, just basic types.
//! Every AST node type is generic and can hold an extra value. This is useful
//! to carry metadata along with the AST (e.g. [Span]).

use crate::util::Span;

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
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StackRef(pub StackId);

/// A reference to a register. Registers can be readonly (in which case the
/// value is a reflection of some other part of state), or read-write, which
/// means the user can read and write freely from/to it.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RegisterRef {
    /// Read-only register that provides the number of elements remaining
    /// in the input buffer
    InputLength,
    /// Read-only register that provides the current length of (i.e. the number
    /// of elements stored in) the referenced stack
    StackLength(StackId),
    /// User-writable register to be used for arbitrary computations
    User(UserRegisterId),
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
    /// Reads one value from the input buffer to a register
    Read(Node<RegisterRef, T>),
    /// Writes a value to the output buffer
    Write(Node<ValueSource<T>, T>),
    /// Sets a register to a value
    Set(Node<RegisterRef, T>, Node<ValueSource<T>, T>),
    /// Adds two values. Puts the result in the first argument.
    Add(Node<RegisterRef, T>, Node<ValueSource<T>, T>),
    /// Subtracts the second value from the first. Puts the result in the
    /// first argument.
    Sub(Node<RegisterRef, T>, Node<ValueSource<T>, T>),
    /// Multiplies the two values. Puts the result in the first argument.
    Mul(Node<RegisterRef, T>, Node<ValueSource<T>, T>),
    /// Compares the last two arguments, and stores the comparison result in
    /// the first register. Result is -1 if the first value is less than the
    /// second, 0 if they are equal, and 1 if the first value is greater. The
    /// result will **never** be any value other than -1, 0, or 1.
    ///
    /// TODO: maybe we should remove this op?
    Cmp(
        Node<RegisterRef, T>,
        Node<ValueSource<T>, T>,
        Node<ValueSource<T>, T>,
    ),
    /// Pushes the value in a register onto the given stack
    Push(Node<ValueSource<T>, T>, Node<StackRef, T>),
    /// Pops the top value off the given stack into a register
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
    }
}
