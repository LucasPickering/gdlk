use crate::ast::{source::LabelDecl, Label, RegisterRef, SpanNode, StackRef};
use failure::Fail;
use serde::Serialize;
use std::{
    fmt::{self, Display, Formatter},
    ops::Try,
};

// TODO re-work these error messages

/// An error that occurs during compilation of a program. The error will be
/// due to a flaw in the program. This indicates a user error, _not_ an internal
/// compiler error. Compiler bugs will always cause a panic.
#[derive(Debug, PartialEq, Fail, Serialize)]
pub enum CompileError {
    /// Failed to parse the program
    #[fail(display = "Parse error: {}", 0)]
    ParseError(String),

    /// Referenced a user register with an invalid identifier
    #[fail(display = "Invalid reference to register @ {}", 0)]
    InvalidRegisterRef(SpanNode<RegisterRef>),

    /// Referenced a stack with an invalid identifier
    #[fail(display = "Invalid reference to stack @ {}", 0)]
    InvalidStackRef(SpanNode<StackRef>),

    /// Tried to write to a read-only register
    #[fail(display = "Cannot write to read-only register @ {}", 0)]
    UnwritableRegister(SpanNode<RegisterRef>),

    /// Defined a label more than once
    #[fail(display = "Duplicate label @ {}", 0)]
    DuplicateLabel(SpanNode<LabelDecl>),

    /// Referenced a label that wasn't defined
    #[fail(display = "Invalid reference to label @ {}", 0)]
    InvalidLabel(SpanNode<Label>),
}

/// An error that occurs during execution of a program. The error will be
/// due to a flaw in the program. This indicates a user error, _not_ a bug in
/// the interpreter. Interpreter bugs will always panic.
#[derive(Debug, PartialEq, Fail, Serialize)]
pub enum RuntimeError {
    /// Tried to push onto stack that is at capacity
    #[fail(display = "Overflow on stack @ {}", 0)]
    StackOverflow(SpanNode<StackRef>),

    /// READ attempted while input is empty
    #[fail(display = "No input available to read")]
    EmptyInput,

    /// POP attempted while stack is empty
    #[fail(display = "Cannot pop from empty stack @ {}", 0)]
    EmptyStack(SpanNode<StackRef>),

    /// Too many cycles in the program
    #[fail(display = "The maximum number of cycles has been reached")]
    TooManyCycles,

    /// Instruction list has been exhausted, program is terminated
    #[fail(display = "Program has terminated, nothing left to execute")]
    ProgramTerminated,
}

/// A collection of compiler errors. We want to show as many errors as possible
/// at compile time, so that the user can see everything wrong with their
/// program at once.
///
/// This holds an `Option<Vec<_>>` instead of just a `Vec` so that we don't
/// have to allocate on the heap until we know we have errors. There are some
/// methods and traits implemented to make it easier to collect errors as you
/// go through a program.
#[derive(Debug, PartialEq, Fail, Serialize)]
pub struct CompileErrors(Option<Vec<CompileError>>);

impl CompileErrors {
    /// Returns an empty set of errors. This will NOT allocate any heap memory.
    pub fn none() -> Self {
        Self(None)
    }

    /// Adds a new error to the collection
    pub fn push(&mut self, error: CompileError) {
        match &mut self.0 {
            None => {
                self.0 = Some(vec![error]);
            }
            Some(errs) => errs.push(error),
        }
    }

    /// Combines this error collection with another, returning a collection with
    /// both sets of errors.
    pub fn chain(mut self, other: Self) -> Self {
        // If `other` has errors:
        if let Some(other_errs) = other.0 {
            match &mut self.0 {
                // We don't have errors, just return the others
                None => return Self(Some(other_errs)),
                // Combine the two collections
                Some(self_errs) => {
                    self_errs.extend(other_errs);
                }
            }
        }
        self
    }
}

// For converting a single error into a collection
impl From<CompileError> for CompileErrors {
    fn from(error: CompileError) -> Self {
        Self(Some(vec![error]))
    }
}

// Implemented so we can use `?` with this type
impl Try for CompileErrors {
    type Ok = ();
    type Error = Self;

    fn into_result(self) -> Result<(), CompileErrors> {
        match self.0 {
            Some(errs) if !errs.is_empty() => Err(Self(Some(errs))),
            _ => Ok(()),
        }
    }

    fn from_ok(_: ()) -> Self {
        Self::none()
    }

    fn from_error(errs: Self) -> Self {
        errs
    }
}

impl Display for CompileErrors {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.0 {
            None => write!(f, "No errors"),
            Some(errors) => {
                // Write each error, separated by a newline
                for (i, error) in errors.iter().enumerate() {
                    // Prefix with a newline for all errors but the first
                    if i > 0 {
                        writeln!(f)?;
                    }
                    write!(f, "{}", error)?;
                }
                Ok(())
            }
        }
    }
}
