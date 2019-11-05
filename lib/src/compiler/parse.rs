use crate::{ast::Ast, compiler::Compiler};
use failure::Error;
use std::io::Read;

impl Compiler<()> {
    /// Parses source code from the given input, into an abstract syntax tree.
    pub fn parse(self, input: &mut impl Read) -> Result<Compiler<Ast>, Error> {
        let mut source_buffer = String::new();
        input.read_to_string(&mut source_buffer)?;
        Ok(Compiler(Ast::Value(source_buffer)))
    }
}
