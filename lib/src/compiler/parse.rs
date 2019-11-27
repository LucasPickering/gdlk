use crate::{ast::Program, compiler::Compiler, error::CompileError};
use std::io::Read;

impl Compiler<()> {
    /// Parses source code from the given input, into an abstract syntax tree.
    pub fn parse(
        self,
        source: &mut impl Read,
    ) -> Result<Compiler<Program>, CompileError> {
        let mut source_buffer = String::new();
        source
            .read_to_string(&mut source_buffer)
            .map_err(|err| CompileError::ParseError(format!("{}", err)))?;
        // TODO real parsing here
        let program = serde_json::from_str(&source_buffer)
            .map_err(|err| CompileError::ParseError(format!("{}", err)))?;
        Ok(Compiler(program))
    }
}
