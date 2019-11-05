use crate::{
    ast::{Ast, WellFormedAst},
    compiler::Compiler,
};
use failure::Error;

impl Compiler<Ast> {
    /// Checks that the program is well-formed. What that actually means remains
    /// to be seen.
    pub fn check_well_formed(self) -> Result<Compiler<WellFormedAst>, Error> {
        let wf_ast = match self.0 {
            Ast::Value(value) => WellFormedAst::Value(value),
        };
        Ok(Compiler(wf_ast))
    }
}
