use crate::{ast::RawProgram, compiler::Compiler};
use failure::Error;
use std::io::Read;

impl Compiler<()> {
    /// Parses source code from the given input, into an abstract syntax tree.
    pub fn parse(
        self,
        input: &mut impl Read,
    ) -> Result<Compiler<RawProgram>, Error> {
        let mut source_buffer = String::new();
        input.read_to_string(&mut source_buffer)?;
        // TODO real parsing here
        let program = serde_json::from_str(&source_buffer)?;
        Ok(Compiler(program))
    }
}
