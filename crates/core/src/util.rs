use nom::Slice;
use nom_locate::LocatedSpan;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Formatter},
    iter,
};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

pub type RawSpan<'a> = LocatedSpan<&'a str>;

/// A definition of a span of source code. This doesn't actually hold the code
/// itself (or any reference to it), it just defines parameters that can be used
/// to find the source span.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Span {
    // TODO make these readonly and camel case in wasm
    /// Distance into the source at which this span starts. Starts at `0`.
    pub offset: usize,
    /// Number of characters that this span includes.
    pub length: usize,
    /// The line number that this span starts on, starting at `1`.
    pub start_line: usize,
    /// The column that this span starts at, starting at `1`.
    pub start_col: usize,
    /// The line number that this span ends on, starting at `1`.
    pub end_line: usize,
    /// The column that this span ends at, starting at `1`.
    pub end_col: usize,
}

impl Span {
    /// Construction a new [Span] from a [RawSpan], using its fragment to
    /// determine the length and end position.
    pub fn from_raw_span(start: &RawSpan) -> Self {
        // Create a new RawSpan to capture the end of the fragment
        let len = start.fragment().len();
        let end = start.slice(len..);

        Self {
            offset: start.location_offset(),
            length: start.fragment().len(),
            start_line: start.location_line() as usize,
            start_col: start.get_column(),
            end_line: end.location_line() as usize,
            end_col: end.get_column(),
        }
    }

    /// Construction a new [Span] from a [RawSpan], but ignore its fragment.
    /// The constructed span will be length `0` and the end line/column will
    /// be the same as the start. Useful when the fragment doesn't convey any
    /// real information (e.g. for errors).
    pub fn from_position(raw_span: &RawSpan) -> Self {
        let line = raw_span.location_line() as usize;
        let col = raw_span.get_column();

        Self {
            offset: raw_span.location_offset(),
            length: 0,
            start_line: line,
            start_col: col,
            end_line: line,
            // +1 so the underlining shows 1 caret
            end_col: col + 1,
        }
    }

    /// Determine if a line number intersects with this span.
    pub fn includes_line(&self, line_num: usize) -> bool {
        self.start_line <= line_num && line_num <= self.end_line
    }

    /// Get the start and end column of this span for a particular line. For the
    /// first line in the span, the start is the span's start column. For the
    /// last line, the end is the span's end column. For any other line, the
    /// start is `1` and the end is the given line length.
    pub fn get_cols_for_line(
        &self,
        line_num: usize,
        line_len: usize,
    ) -> (usize, usize) {
        let start_col = if line_num <= self.start_line {
            self.start_col
        } else {
            1
        };
        let end_col = if line_num >= self.end_line {
            self.end_col
        } else {
            line_len
        };
        (start_col, end_col)
    }

    /// Find the spanned portion of source within the full source code. Returns
    /// a sub-slice of the given string that corresponds to this span.
    pub fn get_source_slice<'a>(&self, src: &'a str) -> &'a str {
        &src[self.offset..(self.offset + self.length)]
    }
}

pub fn fmt_src_highlights(
    f: &mut Formatter<'_>,
    span: &Span,
    src: &str,
) -> fmt::Result {
    let margin = "   ";
    let separator = " | ";

    // Span's line numbers start at 1, so include a dummy line at the
    // beginning here to make them line up
    let lines: Vec<&str> = iter::once("").chain(src.lines()).collect();
    writeln!(f)?; // Blank line
    writeln!(f, "{}{}", margin, separator)?;

    // Print the the source span, plus an extra line before and after
    let highlight_start_line = usize::max(span.start_line - 1, 1);
    let highlight_end_line = usize::min(span.end_line + 1, lines.len() - 1);
    for (i, line) in lines[highlight_start_line..=highlight_end_line]
        .iter()
        .enumerate()
    {
        let line_num = highlight_start_line + i;
        writeln!(f, "{:>3}{}{}", line_num, separator, line)?;

        // If this line is actually in the span, do some underlining
        if span.includes_line(line_num) {
            // Underline the spanned columns with ^^^
            let (start_col, end_col) =
                span.get_cols_for_line(line_num, line.len());
            writeln!(
                f,
                "{}{}{}{}",
                margin,
                separator,
                iter::repeat(" ")
                    .take(start_col - 1)
                    .collect::<Vec<_>>()
                    .join(""),
                iter::repeat("^")
                    .take(end_col - start_col)
                    .collect::<Vec<_>>()
                    .join("")
            )?;
        }
    }
    writeln!(f, "{}{}", margin, separator)?;

    Ok(())
}

/// Macro that can wrap any body, and only executes the body if we are running
/// in debug mode. Debug mode is enabled by setting the environment variable
/// `DEBUG=true`. This compiles away to nothing when --release is used.
///
/// ```
/// use gdlk::debug;
/// debug!(println!("Hello!"));
/// ```
#[macro_export]
macro_rules! debug {
    ($arg:expr) => {
        #[cfg(debug_assertions)]
        {
            if let Ok(debug_val) = std::env::var("DEBUG") {
                if ["1", "t", "true"]
                    .contains(&debug_val.to_lowercase().as_str())
                {
                    $arg
                }
            }
        }
    };
}

// Only needed in tests
impl PartialEq<Span> for Span {
    fn eq(&self, other: &Self) -> bool {
        // Skip offset and length, just to make testing a bit easier
        self.start_line == other.start_line
            && self.start_col == other.start_col
            && self.end_line == other.end_line
            && self.end_col == other.end_col
    }
}
