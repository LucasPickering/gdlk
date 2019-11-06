#![deny(clippy::all)]

use crate::compiler::Compiler;
use failure::Error;
use std::io::{Read, Write};

mod ast;
mod compiler;
mod error;

/// Reads source code from the given [`Read`](Read), compiles it, and outputs
/// the corresponding target language code as a string.
///
/// In the future, this should take in multiple input files, but that's a
/// problem for future me.
pub fn compile(
    input: &mut impl Read,
    output: &mut impl Write,
) -> Result<(), Error> {
    // Run all the pipeline stages in order. If any stage fails, bail out.
    Compiler::new()
        .parse(input)?
        .check_well_formed()?
        .generate_code(output)
}
