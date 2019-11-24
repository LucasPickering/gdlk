use crate::{ast::Program, compiler::Compiler};
use failure::Fallible;
use std::io::Read;

impl Compiler<()> {
    /// Parses source code from the given input, into an abstract syntax tree.
    pub fn parse(self, source: &mut impl Read) -> Fallible<Compiler<Program>> {
        let mut source_buffer = String::new();
        source.read_to_string(&mut source_buffer)?;
        // TODO real parsing here
        let program = serde_json::from_str(&source_buffer)?;
        Ok(Compiler(program))
    }
}
