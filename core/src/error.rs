//! All error-related GDLK types.

#[cfg(feature = "wasm")]
use crate::ast::wasm::SourceElement;
use crate::util::{self, Span};
use serde::Serialize;
use std::fmt::{self, Debug, Display, Formatter};
use thiserror::Error;

/// A trait for any error that originates in source code. [SourceError]s rely on
/// having source code present in order to display themselves.
pub trait SourceError: 'static + Send + Sync + Debug + Serialize {
    /// A simple type label for this error, e.g. `"syntax"` or `"runtime"`.
    fn type_label(&self) -> &'static str;

    /// Format this error into a simple message. `spanned_src` is the slice of
    /// the source code that corresponds to this error's [Span]. This needs to
    /// be provided by the caller in order to create a proper error message.
    fn fmt_msg(&self, f: &mut Formatter<'_>, spanned_src: &str) -> fmt::Result;
}

/// An error that occurs during compilation of a program. The error will be
/// due to a flaw in the program. This indicates a user error, _not_ an internal
/// compiler error. Compiler bugs will always cause a panic.
#[derive(Copy, Clone, Debug, Serialize)]
pub enum CompileError {
    /// Failed to parse the program because of a syntax error. `expected` is
    /// the name of the type of element that was expected where the error
    /// occured.
    Syntax { expected: &'static str },
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
    fn type_label(&self) -> &'static str {
        match self {
            Self::Syntax { .. } => "Syntax",
            _ => "Validation",
        }
    }

    fn fmt_msg(&self, f: &mut Formatter<'_>, spanned_src: &str) -> fmt::Result {
        match self {
            // the source span for syntax errors is just the remaining source,
            // so not very helpful
            Self::Syntax { expected } => write!(f, "Expected {}", expected,),
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
#[derive(Copy, Clone, Debug, Serialize)]
pub enum RuntimeError {
    /// DIV attempted with a zero divisor
    DivideByZero,
    /// READ attempted while input is empty
    EmptyInput,
    /// PUSH attemped onto a stack that is at capacity
    StackOverflow,
    /// POP attempted from an empty stack
    EmptyStack,
    /// Execution attempted after the program has hit the CPU cycle limit
    TooManyCycles,
}

impl SourceError for RuntimeError {
    fn type_label(&self) -> &'static str {
        "Runtime"
    }

    fn fmt_msg(&self, f: &mut Formatter<'_>, spanned_src: &str) -> fmt::Result {
        match self {
            Self::DivideByZero => write!(f, "Divide by zero"),
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
#[derive(Clone, Debug, Error, Serialize)]
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

    pub fn span(&self) -> Span {
        self.span
    }
}

impl<E: SourceError> Display for SourceErrorWrapper<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} error at {}:{}: ",
            self.error.type_label(),
            self.span.start_line,
            self.span.start_col,
        )?;
        self.error.fmt_msg(f, &self.spanned_source)?;
        Ok(())
    }
}

// This makes it a bit easier to send errors out to wasm
#[cfg(feature = "wasm")]
impl<E: SourceError> From<&SourceErrorWrapper<E>> for SourceElement {
    fn from(error: &SourceErrorWrapper<E>) -> Self {
        SourceElement {
            text: error.to_string(),
            span: error.span(),
        }
    }
}

/// A wrapper around of a collection of errors. This holds the errors as well as
/// the source code, and can be used to render associated source code with each
/// error.
#[derive(Clone, Debug, Error, Serialize)]
pub struct WithSource<E: SourceError> {
    errors: Vec<SourceErrorWrapper<E>>,
    #[serde(skip)]
    source_code: String,
}

impl<E: SourceError> WithSource<E> {
    /// Wrap a collection of errors with its source code.
    pub(crate) fn new(
        errors: impl IntoIterator<Item = SourceErrorWrapper<E>>,
        source: String,
    ) -> Self {
        Self {
            errors: errors.into_iter().collect(),
            source_code: source,
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
                util::fmt_src_highlights(f, &error.span, &self.source_code)?;
            }
        }
        Ok(())
    }
}
