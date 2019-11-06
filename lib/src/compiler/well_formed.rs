use crate::{
    ast::{
        RawBody, RawInstruction, RawProgram, WellFormedBody,
        WellFormedInstruction, WellFormedProgram,
    },
    compiler::Compiler,
    env::Env,
};
use failure::Error;

fn check_body_well_formed(
    body: RawBody,
    env: &mut Env,
) -> Result<WellFormedBody, Error> {
    let mut result = Vec::new();
    for instr in body {
        let wf_instr = match instr {
            RawInstruction::Set(value) => WellFormedInstruction::Set(value),
            RawInstruction::Create(ident) => {
                // Insert the ident to the env. If insert returns false, it
                // means the ident was already in the env, so return an error.
                env.declare(&ident)?;
                WellFormedInstruction::Create(ident)
            }
            RawInstruction::Destroy(ident) => {
                env.verify_exists(&ident)?;
                WellFormedInstruction::Destroy(ident)
            }
            RawInstruction::Push(ident) => {
                env.verify_exists(&ident)?;
                WellFormedInstruction::Push(ident)
            }
            RawInstruction::Pop(ident) => {
                env.verify_exists(&ident)?;
                WellFormedInstruction::Pop(ident)
            }
        };
        result.push(wf_instr)
    }
    Ok(result)
}

impl Compiler<RawProgram> {
    /// Checks that the program is well-formed. What that actually means remains
    /// to be seen.
    pub fn check_well_formed(
        self,
    ) -> Result<Compiler<WellFormedProgram>, Error> {
        Ok(Compiler(WellFormedProgram {
            body: check_body_well_formed(self.0.body, &mut Env::new())?,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assert_error, ast::RawValue};

    #[test]
    fn test_no_errors() {
        let compiler = Compiler(RawProgram {
            body: vec![
                RawInstruction::Create("id".into()),
                RawInstruction::Set(RawValue::Int(5)),
                RawInstruction::Push("id".into()),
                RawInstruction::Pop("id".into()),
                RawInstruction::Destroy("id".into()),
            ],
        });
        let result = compiler.check_well_formed();
        assert!(result.is_ok());
    }

    #[test]
    fn test_duplicate_ident() {
        let compiler = Compiler(RawProgram {
            body: vec![
                RawInstruction::Create("id".into()),
                RawInstruction::Create("id".into()),
            ],
        });
        assert_error!(compiler.check_well_formed(), "Duplicate identifier");
    }

    #[test]
    fn test_unknown_ident() {
        let compiler = Compiler(RawProgram {
            body: vec![RawInstruction::Push("id".into())],
        });
        assert_error!(compiler.check_well_formed(), "Unknown identifier");
    }
}
