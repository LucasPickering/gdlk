use crate::{
    ast::{compiled, source, Label},
    Compiler,
};
use std::collections::HashMap;

/// Build a mapping of all labels to the their instruction indexes. The indexes
/// exclude the labels themselves.
fn map_labels(body: &[source::Statement]) -> HashMap<Label, isize> {
    let mut label_map: HashMap<Label, isize> = HashMap::new();
    for (i, stmt) in body.iter().enumerate() {
        if let source::Statement::Label(label) = stmt {
            // Need subtract 1 for each label above us (because they will be
            // removed from the list)
            label_map.insert(label.into(), (i - label_map.len()) as isize);
        }
    }
    label_map
}

impl Compiler<source::Program> {
    /// Removes labels from the source, replacing their references with relative
    /// index offsets.
    pub fn delabel(self) -> Compiler<compiled::Program> {
        let label_map = map_labels(&self.0.body);

        let instructions: Vec<compiled::Instruction> = self
            .0
            .body
            .into_iter()
            // Need to filter FIRST so labels don't get tracked in the indexes
            .filter(|stmt| match stmt {
                source::Statement::Label(_) => false,
                _ => true,
            })
            .enumerate()
            .map(|(i, stmt)| match stmt {
                source::Statement::Label(_) => unreachable!(),
                source::Statement::Operator(op) => {
                    compiled::Instruction::Operator(op)
                }
                source::Statement::Jump(jump, label) => {
                    compiled::Instruction::Jump(
                        jump,
                        // Get a relative offset to the label
                        // the program would have to be VERY big for this to
                        // break
                        *label_map.get(&label).unwrap() - i as isize,
                    )
                }
            })
            .collect();
        Compiler(compiled::Program { instructions })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Jump, Operator, RegisterRef};

    #[test]
    fn test_delabel() {
        let body = vec![
            source::Statement::Label("START".into()),
            source::Statement::Jump(Jump::Jmp, "START".into()),
            source::Statement::Operator(Operator::Read(RegisterRef::User(0))),
            source::Statement::Jump(Jump::Jmp, "START".into()),
            source::Statement::Jump(Jump::Jmp, "END".into()),
            source::Statement::Operator(Operator::Read(RegisterRef::User(0))),
            source::Statement::Label("END".into()),
        ];
        let compiler = Compiler(source::Program { body });
        assert_eq!(
            compiler.delabel().0.instructions,
            vec![
                compiled::Instruction::Jump(Jump::Jmp, 0),
                compiled::Instruction::Operator(Operator::Read(
                    RegisterRef::User(0)
                )),
                compiled::Instruction::Jump(Jump::Jmp, -2),
                compiled::Instruction::Jump(Jump::Jmp, 2),
                compiled::Instruction::Operator(Operator::Read(
                    RegisterRef::User(0)
                )),
            ]
        );
    }
}
