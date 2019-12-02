use crate::lang::{
    ast::{Instr, MachineInstr, Program},
    Compiler,
};
use std::convert::TryInto;

/// Desugar/flatten a single instruction. `tag` should be unique to this
/// instruction, so that unique labels can be generated where necessary.
fn desugar_instr(instr: Instr) -> Vec<MachineInstr> {
    match instr {
        Instr::Read => vec![MachineInstr::Read],
        Instr::Write => vec![MachineInstr::Write],
        Instr::Set(value) => vec![MachineInstr::Set(value)],
        Instr::Push(stack_id) => vec![MachineInstr::Push(stack_id)],
        Instr::Pop(stack_id) => vec![MachineInstr::Pop(stack_id)],
        Instr::If(body) => {
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
            // If workspace == 0, jump over all the instructions inside the IF
            let jump = MachineInstr::Jez(body_len + 1);
            desugared_body.insert(0, jump); // Add the jump before the IF
            desugared_body
        }
        Instr::While(body) => {
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

            // Used to skip the initial loop iteration when workspace == 0
            let prejump = MachineInstr::Jez(body_len + 2);
            // Used to go back to the top of the loop when workspace != 0
            let postjump = MachineInstr::Jnz(-body_len);

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

    #[test]
    fn test_if() {
        let compiler = Compiler(Program {
            body: vec![
                Instr::Set(0),
                Instr::If(vec![Instr::Set(1)]),
                Instr::Set(2),
            ],
        });
        assert_eq!(
            compiler.desugar().0,
            vec![
                MachineInstr::Set(0),
                MachineInstr::Jez(2),
                MachineInstr::Set(1),
                MachineInstr::Set(2),
            ]
        )
    }

    #[test]
    fn test_while() {
        let compiler = Compiler(Program {
            body: vec![
                Instr::Set(0),
                Instr::While(vec![Instr::Set(1)]),
                Instr::Set(2),
            ],
        });
        assert_eq!(
            compiler.desugar().0,
            vec![
                MachineInstr::Set(0),
                MachineInstr::Jez(3),
                MachineInstr::Set(1),
                MachineInstr::Jnz(-1),
                MachineInstr::Set(2),
            ]
        )
    }
}
