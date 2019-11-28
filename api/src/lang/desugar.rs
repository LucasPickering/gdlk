use crate::lang::{
    ast::{DietInstr, Instr, Program},
    Compiler,
};
use std::iter;

/// Desugar/flatten a single instruction. `tag` should be unique to this
/// instruction, so that unique labels can be generated where necessary.
fn desugar_instr(instr: Instr, tag: usize) -> Vec<DietInstr> {
    match instr {
        Instr::Read => vec![DietInstr::Read],
        Instr::Write => vec![DietInstr::Write],
        Instr::Set(value) => vec![DietInstr::Set(value)],
        Instr::Push(stack_id) => vec![DietInstr::Push(stack_id)],
        Instr::Pop(stack_id) => vec![DietInstr::Pop(stack_id)],
        Instr::If(body) => {
            let label = format!("if_end_{}", tag);
            iter::once(DietInstr::Jez(label.clone()))
                .chain(desugar_instrs(body, tag + 1).into_iter())
                .chain(iter::once(DietInstr::Label(label)))
                .collect()
        }
        Instr::While(body) => {
            let start_label = format!("while_start_{}", tag);
            let end_label = format!("while_end_{}", tag);
            // If workspace == 0, skip the WHILE
            iter::once(DietInstr::Jez(end_label.clone()))
                // A label to mark the start of the WHILE body
                .chain(iter::once(DietInstr::Label(start_label.clone())))
                // The WHILE body
                .chain(desugar_instrs(body, tag + 1).into_iter())
                // If workspace != 0 still, repeat
                .chain(iter::once(DietInstr::Jnz(start_label)))
                // A label used to skip the WHILE body
                .chain(iter::once(DietInstr::Label(end_label)))
                .collect()
        }
    }
}

/// Desugar/flatten a series of instructions. `tag` should be monotonically
/// increasing, such that each individual instruction is desugared with a
/// unique tag. This is used to generate unique labels.
fn desugar_instrs(instrs: Vec<Instr>, tag: usize) -> Vec<DietInstr> {
    instrs
        .into_iter()
        .enumerate()
        .map(|(idx, instr)| desugar_instr(instr, tag + idx))
        .flatten()
        .collect()
}

impl Compiler<Program> {
    /// Desugars and flattens the nested AST into a flat AST, so that it can
    /// more easily executed. Nested instructions such as IF and WHILE get
    /// replaced with jumps. Most remain untouched by this
    pub fn desugar(self) -> Compiler<Vec<DietInstr>> {
        Compiler(desugar_instrs(self.0.body, 0))
    }
}
