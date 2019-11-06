use crate::{ast::WellFormedProgram, compiler::Compiler};
use failure::Error;
use std::io::Write;

impl Compiler<WellFormedProgram> {
    /// Generates code for the target language, and writes it to the given
    /// output destination. This is the final compiler step.
    pub fn generate_code(self, output: &mut impl Write) -> Result<(), Error> {
        let json = serde_json::to_string(&self.0)?;
        output.write_all(json.as_bytes())?;
        Ok(())
    }
}
