use crate::ast::Program;
use failure::Fallible;
use std::io::Read;

impl Program {
    /// Parses source code from the given input, into an abstract syntax tree.
    pub fn parse(source: &mut impl Read) -> Fallible<Self> {
        let mut source_buffer = String::new();
        source.read_to_string(&mut source_buffer)?;
        // TODO real parsing here
        let program = serde_json::from_str(&source_buffer)?;
        Ok(program)
    }
}
