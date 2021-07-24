use crate::{
    ast::{
        compiled::{self},
        source::{self, LabelDecl, Statement},
        Instruction, Node,
    },
    util::Span,
    Compiler, ProgramStats,
};
use std::collections::HashMap;

impl Compiler<(source::Program<Span>, ProgramStats)> {
    /// Removes labels from the source, and pull them into a separate symbol
    /// table. The symbol table will map each label to its location in the
    /// program. The location will be an index into the vector of instructions
    /// that this function generates for the new program.
    pub(crate) fn delabel(self) -> Compiler<compiled::Program<Span>> {
        let body = self.ast.0.body;
        let stats = self.ast.1;

        // Do a pass over the instructions and collect two things:
        // 1. A mapping of label:index, showing where a label exists in code
        // 2. All instructions (i.e. all statements *except* labels)
        // The label indexes will refer to the resulting list of *instructions*,
        // NOT the input list of *statements*
        let mut symbol_table = HashMap::new();
        let mut instructions: Vec<Node<Instruction<_>, _>> = Vec::new();
        for statement in body {
            match statement.0 {
                Statement::Label(Node(LabelDecl(label), _)) => {
                    symbol_table.insert(label, instructions.len());
                }
                Statement::Instruction(instruction_node) => {
                    instructions.push(instruction_node);
                }
            }
        }

        Compiler {
            source: self.source,
            hardware_spec: self.hardware_spec,
            // Stats won't change at this point, just forward them down
            ast: compiled::Program {
                instructions,
                symbol_table,
                stats,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ast::{Instruction, RegisterRef},
        models::HardwareSpec,
    };
    use std::collections::HashSet;

    #[test]
    fn test_delabel() {
        // Dummy span for comparisons
        let span = Span {
            offset: 0,
            length: 0,
            start_line: 0,
            start_col: 0,
            end_line: 0,
            end_col: 0,
        };
        let body = vec![
            Node(
                Statement::Label(Node(LabelDecl("START".into()), span)),
                span,
            ),
            Node(
                Statement::Instruction(Node(
                    Instruction::Jmp(Node("START".into(), span)),
                    span,
                )),
                span,
            ),
            Node(
                Statement::Instruction(Node(
                    Instruction::Read(Node(RegisterRef::User(0), span)),
                    span,
                )),
                span,
            ),
            Node(
                Statement::Instruction(Node(
                    Instruction::Jmp(Node("START".into(), span)),
                    span,
                )),
                span,
            ),
            Node(
                Statement::Instruction(Node(
                    Instruction::Jmp(Node("END".into(), span)),
                    span,
                )),
                span,
            ),
            Node(
                Statement::Instruction(Node(
                    Instruction::Read(Node(RegisterRef::User(0), span)),
                    span,
                )),
                span,
            ),
            Node(Statement::Label(Node(LabelDecl("END".into()), span)), span),
        ];
        let empty_stats = ProgramStats {
            referenced_registers: HashSet::new(),
            referenced_stacks: HashSet::new(),
        };
        let compiler = Compiler {
            source: "".into(),
            hardware_spec: HardwareSpec::default(),
            ast: (source::Program { body }, empty_stats),
        };
        assert_eq!(
            compiler.delabel().ast.instructions,
            vec![
                Node(Instruction::Jmp(Node("START".into(), span)), span),
                Node(Instruction::Read(Node(RegisterRef::User(0), span)), span),
                Node(Instruction::Jmp(Node("START".into(), span)), span),
                Node(Instruction::Jmp(Node("END".into(), span)), span),
                Node(
                    Instruction::Read(Node(RegisterRef::User(0,), span)),
                    span
                )
            ]
        );
    }
}
