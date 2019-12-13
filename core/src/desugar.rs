use crate::{
    ast::{Instr, MachineInstr, Program},
    Compiler,
};
use std::convert::TryInto;

/// Desugar/flatten a single instruction. `tag` should be unique to this
/// instruction, so that unique labels can be generated where necessary.
fn desugar_instr(instr: Instr) -> Vec<MachineInstr> {
    match instr {
        Instr::Operator(op) => vec![MachineInstr::Operator(op)],

        Instr::If(reg_id, body) => {
            // This conversion looks like:
            //
            // IF {
            //     SET 1
            //     SET 2
            // }
            // SET 3
            //
            // 0: JEZ 3 --+
            // 1: SET 1   |
            // 2: SET 2   |
            // 4: SET 3 <-+

            // Desugar the body of the IF
            let mut desugared_body = desugar_instrs(body);
            // TODO make this unwrap safe
            let body_len: i32 = desugared_body.len().try_into().unwrap();
            // If register == 0, jump over all the instructions inside the IF
            let jump = MachineInstr::Jez(body_len + 1, reg_id);
            desugared_body.insert(0, jump); // Add the jump before the IF
            desugared_body
        }
        Instr::While(reg_id, body) => {
            // This conversion looks like:
            //
            // WHILE {
            //     SET 1
            //     SET 2
            // }
            // SET 3
            //
            // 0: JEZ 4 ----+
            // 1: SET 1 <-+ |
            // 2: SET 2   | |
            // 3: JNZ -2 -+ |
            // 4: SET 3 <---+

            // Desugar the body of the WHILE
            let mut desugared_body = desugar_instrs(body);
            // TODO make this unwrap safe
            let body_len: i32 = desugared_body.len().try_into().unwrap();

            // Used to skip the initial loop iteration when register == 0
            let prejump = MachineInstr::Jez(body_len + 2, reg_id);
            // Used to go back to the top of the loop when register != 0
            let postjump = MachineInstr::Jnz(-body_len, reg_id);

            desugared_body.insert(0, prejump);
            desugared_body.push(postjump);
            desugared_body
        }
    }
}

/// Desugar/flatten a series of instructions. `tag` should be monotonically
/// increasing, such that each individual instruction is desugared with a
/// unique tag. This is used to generate unique labels.
fn desugar_instrs(instrs: Vec<Instr>) -> Vec<MachineInstr> {
    instrs.into_iter().map(desugar_instr).flatten().collect()
}

impl Compiler<Program> {
    /// Desugars and flattens the nested AST into a flat AST, so that it can
    /// more easily executed. Nested instructions such as IF and WHILE get
    /// replaced with jumps. Most remain untouched by this
    pub fn desugar(self) -> Compiler<Vec<MachineInstr>> {
        Compiler(desugar_instrs(self.0.body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Operator, RegisterRef, ValueSource};

    #[test]
    fn test_if() {
        let compiler = Compiler(Program {
            body: vec![
                Instr::Operator(Operator::Set(
                    RegisterRef::User(0),
                    ValueSource::Const(0),
                )),
                Instr::If(
                    RegisterRef::User(0),
                    vec![Instr::Operator(Operator::Set(
                        RegisterRef::User(0),
                        ValueSource::Const(1),
                    ))],
                ),
                Instr::Operator(Operator::Set(
                    RegisterRef::User(0),
                    ValueSource::Const(2),
                )),
            ],
        });
        assert_eq!(
            compiler.desugar().0,
            vec![
                MachineInstr::Operator(Operator::Set(
                    RegisterRef::User(0),
                    ValueSource::Const(0)
                )),
                MachineInstr::Jez(2, RegisterRef::User(0)),
                MachineInstr::Operator(Operator::Set(
                    RegisterRef::User(0),
                    ValueSource::Const(1)
                )),
                MachineInstr::Operator(Operator::Set(
                    RegisterRef::User(0),
                    ValueSource::Const(2)
                )),
            ]
        )
    }

    #[test]
    fn test_while() {
        let compiler = Compiler(Program {
            body: vec![
                Instr::Operator(Operator::Set(
                    RegisterRef::User(0),
                    ValueSource::Const(0),
                )),
                Instr::While(
                    RegisterRef::User(0),
                    vec![Instr::Operator(Operator::Set(
                        RegisterRef::User(0),
                        ValueSource::Const(1),
                    ))],
                ),
                Instr::Operator(Operator::Set(
                    RegisterRef::User(0),
                    ValueSource::Const(2),
                )),
            ],
        });
        assert_eq!(
            compiler.desugar().0,
            vec![
                MachineInstr::Operator(Operator::Set(
                    RegisterRef::User(0),
                    ValueSource::Const(0)
                )),
                MachineInstr::Jez(3, RegisterRef::User(0)),
                MachineInstr::Operator(Operator::Set(
                    RegisterRef::User(0),
                    ValueSource::Const(1)
                )),
                MachineInstr::Jnz(-1, RegisterRef::User(0)),
                MachineInstr::Operator(Operator::Set(
                    RegisterRef::User(0),
                    ValueSource::Const(2)
                )),
            ]
        )
    }
}
