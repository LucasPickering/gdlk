//! All error-related GDLK types.

use crate::util::{self, Span};
use failure::Fail;
use serde::Serialize;
use std::fmt::{self, Debug, Display, Formatter};

/// A trait for any error that originates in source code. [SourceError]s rely on
/// having source code present in order to display themselves.
pub trait SourceError: 'static + Send + Sync + Debug + Serialize {
    /// Format this error into a simple message. `spanned_src` is the slice of
    /// the source code that corresponds to this error's [Span]. This needs to
    /// be provided by the caller in order to create a proper error message.
    fn fmt_msg(&self, f: &mut Formatter<'_>, spanned_src: &str) -> fmt::Result;
}

/// An error that occurs during compilation of a program. The error will be
/// due to a flaw in the program. This indicates a user error, _not_ an internal
/// compiler error. Compiler bugs will always cause a panic.
#[derive(Debug, Serialize)]
pub enum CompileError {
    /// Failed to parse the program
    ParseError(String),
    /// Referenced a user register with an invalid identifier
    InvalidRegisterRef,
    /// Referenced a stack with an invalid identifier
    InvalidStackRef,
    /// Tried to write to a read-only register
    UnwritableRegister,
    /// Defined the same label more than once
    DuplicateLabel { original: Span },
    /// Referenced a label that wasn't defined
    InvalidLabel,
}

impl SourceError for CompileError {
    fn fmt_msg(&self, f: &mut Formatter<'_>, spanned_src: &str) -> fmt::Result {
        match self {
            Self::ParseError(err) => write!(f, "Parse error: {}", err),
            Self::InvalidRegisterRef => {
                write!(f, "Invalid reference to register `{}`", spanned_src)
            }
            Self::InvalidStackRef => {
                write!(f, "Invalid reference to stack `{}`", spanned_src)
            }
            Self::UnwritableRegister => write!(
                f,
                "Cannot write to read-only register `{}`",
                spanned_src
            ),
            Self::DuplicateLabel {
                original: original_span,
            } => write!(
                f,
                "Duplicate decalaration of label `{}`, \
                    originally defined on line {}",
                spanned_src, original_span.start_line,
            ),
            Self::InvalidLabel => {
                write!(f, "Invalid reference to label `{}`", spanned_src)
            }
        }
    }
}

/// An error that occurs during execution of a program. The error will be
/// due to a flaw in the program. This indicates a user error, _not_ a bug in
/// the interpreter. Interpreter bugs will always panic.
#[derive(Debug, Serialize)]
pub enum RuntimeError {
    /// READ attempted while input is empty
    EmptyInput,
    /// Tried to push onto stack that is at capacity
    StackOverflow,
    /// POP attempted while stack is empty
    EmptyStack,
    /// Too many cycles in the program
    TooManyCycles,
}

impl SourceError for RuntimeError {
    fn fmt_msg(&self, f: &mut Formatter<'_>, spanned_src: &str) -> fmt::Result {
        match self {
            Self::StackOverflow => {
                write!(f, "Overflow on stack `{}`", spanned_src)
            }
            Self::EmptyInput => {
                write!(f, "Read attempted while input is empty")
            }
            Self::EmptyStack => {
                write!(f, "Cannot pop from empty stack `{}`", spanned_src)
            }
            Self::TooManyCycles => write!(
                f,
                "Maximum number of cycles reached, \
                cannot execute instruction `{}`",
                spanned_src
            ),
        }
    }
}

/// A wrapper around a [SourceError], that holds some extra data:
/// - The [Span] of the source code that caused the error
/// - The offending chunk of source code itself
///
/// This type on its own can be formatted, without any external data.
#[derive(Debug, Fail, Serialize)]
pub struct SourceErrorWrapper<E: SourceError> {
    error: E,
    span: Span,
    spanned_source: String,
}

impl<E: SourceError> SourceErrorWrapper<E> {
    pub fn new(error: E, span: Span, src: &str) -> Self {
        Self {
            error,
            span,
            spanned_source: span.get_source_slice(src).into(),
        }
    }
}

impl<E: SourceError> Display for SourceErrorWrapper<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Error on line {}: ", self.span.start_line)?;
        self.error.fmt_msg(f, &self.spanned_source)?;
        Ok(())
    }
}

/// A wrapper around of a collection of errors. This holds the errors as well as
/// the source code, and can be used to render associated source code with each
/// error.
#[derive(Debug, Fail, Serialize)]
pub struct WithSource<E: SourceError> {
    errors: Vec<SourceErrorWrapper<E>>,
    #[serde(skip)]
    source: String,
}

impl<E: SourceError> WithSource<E> {
    /// Wrap a collection of errors with its source code.
    pub(crate) fn new(
        errors: impl IntoIterator<Item = SourceErrorWrapper<E>>,
        source: String,
    ) -> Self {
        Self {
            errors: errors.into_iter().collect(),
            source,
        }
    }

    /// Get a reference to the errors wrapped by this type.
    pub fn errors(&self) -> &[SourceErrorWrapper<E>] {
        &self.errors
    }
}

impl<E: SourceError> Display for WithSource<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Write each error, separated by a newline
        for (i, error) in self.errors.iter().enumerate() {
            // Prefix with a newline for all errors but the first
            if i > 0 {
                writeln!(f)?; // just a newline
            }

            write!(f, "{}", error)?;
            if f.alternate() {
                util::fmt_src_highlights(f, &error.span, &self.source)?;
            }
        }
        Ok(())
    }
}
