use crate::{
    ast::{Instruction, Label, SourceProgram, SourceStatement},
    Compiler,
};
use std::collections::HashMap;

/// Build a mapping of all labels to the their instruction indexes. The indexes
/// exclude the labels themselves.
fn map_labels(body: &[SourceStatement]) -> HashMap<Label, isize> {
    let mut label_map: HashMap<Label, isize> = HashMap::new();
    for (i, stmt) in body.iter().enumerate() {
        if let SourceStatement::Label(label) = stmt {
            // Need subtract 1 for each label above us (because they will be
            // removed from the list)
            label_map.insert(label.into(), (i - label_map.len()) as isize);
        }
    }
    label_map
}

impl Compiler<SourceProgram> {
    /// Removes labels from the source, replacing their references with relative
    /// index offsets.
    pub fn delabel(self) -> Compiler<Vec<Instruction>> {
        let label_map = map_labels(&self.0.body);

        let instrs: Vec<Instruction> = self
            .0
            .body
            .into_iter()
            // Need to filter FIRST so labels don't get tracked in the indexes
            .filter(|stmt| match stmt {
                SourceStatement::Label(_) => false,
                _ => true,
            })
            .enumerate()
            .map(|(i, stmt)| match stmt {
                SourceStatement::Label(_) => panic!("Shouldn't get here!"),
                SourceStatement::Operator(op) => Instruction::Operator(op),
                SourceStatement::Jump(jump, label) => Instruction::Jump(
                    jump,
                    // Get a relative offset to the label
                    // the program would have to be VERY big for this to break
                    *label_map.get(&label).unwrap() - i as isize,
                ),
            })
            .collect::<Vec<Instruction>>();
        Compiler(instrs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Jump, Operator, RegisterRef};

    #[test]
    fn test_delabel() {
        let body = vec![
            SourceStatement::Label("START".into()),
            SourceStatement::Jump(Jump::Jmp, "START".into()),
            SourceStatement::Operator(Operator::Read(RegisterRef::User(0))),
            SourceStatement::Jump(Jump::Jmp, "START".into()),
            SourceStatement::Jump(Jump::Jmp, "END".into()),
            SourceStatement::Operator(Operator::Read(RegisterRef::User(0))),
            SourceStatement::Label("END".into()),
        ];
        let compiler = Compiler(SourceProgram { body });
        assert_eq!(
            compiler.delabel().0,
            vec![
                Instruction::Jump(Jump::Jmp, 0),
                Instruction::Operator(Operator::Read(RegisterRef::User(0))),
                Instruction::Jump(Jump::Jmp, -2),
                Instruction::Jump(Jump::Jmp, 2),
                Instruction::Operator(Operator::Read(RegisterRef::User(0))),
            ]
        );
    }
}
