use crate::{ast::WellFormedAst, compiler::Compiler};
use failure::Error;
use std::io::Write;

impl Compiler<WellFormedAst> {
    /// Generates code for the target language, and writes it to the given
    /// output destination. This is the final compiler step.
    pub fn generate_code(self, output: &mut impl Write) -> Result<(), Error> {
        output.write_all(format!("{:?}", &self.0).as_bytes())?;
        Ok(())
    }
}
