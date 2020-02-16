use crate::{
    ast::{
        compiled::{Instruction, Program as CompiledProgram},
        source::{LabelDecl, Program as SourceProgram, Statement},
        Label, Node, Span, SpanNode,
    },
    Compiler,
};
use std::collections::HashMap;

/// Build a mapping of all labels to the their instruction indexes. The indexes
/// exclude the labels themselves.
fn map_labels(body: &[SpanNode<Statement<Span>>]) -> HashMap<Label, isize> {
    let mut label_map: HashMap<Label, isize> = HashMap::new();
    for (i, stmt) in body.iter().enumerate() {
        if let Node(Statement::Label(Node(LabelDecl(label), _)), _) = stmt {
            // Need subtract 1 for each label above us (because they will be
            // removed from the list)
            label_map.insert(label.clone(), (i - label_map.len()) as isize);
        }
    }
    label_map
}

/// Helper that maps one source instruction to a compiled instruction. Meant
/// to be passed to a .map() for an iter that has .enumerate() on it.
fn map_statement(
    label_map: &HashMap<Label, isize>,
    i: usize,
    stmt_node: SpanNode<Statement<Span>>,
) -> SpanNode<Instruction<Span>> {
    stmt_node.map(|stmt| match stmt {
        Statement::Label(_) => unreachable!(),
        Statement::Operator(op) => Instruction::Operator(op),
        Statement::Jump(jump, Node(label, _)) => {
            Instruction::Jump(
                jump,
                // Get a relative offset to the label. The program would
                // have to be VERY big for this to break.
                *label_map.get(&label).unwrap() - i as isize,
            )
        }
    })
}

impl Compiler<SourceProgram<Span>> {
    /// Removes labels from the source, replacing their references with relative
    /// index offsets.
    pub fn delabel(self) -> Compiler<CompiledProgram<Span>> {
        let label_map = map_labels(&self.0.body);

        let instructions: Vec<Node<Instruction<_>, _>> = self
            .0
            .body
            .into_iter()
            // Need to filter FIRST so labels don't get tracked in the
            // indexes
            .filter(|stmt_node| match stmt_node {
                Node(Statement::Label(_), _) => false,
                _ => true,
            })
            .enumerate()
            .map(|(i, stmt_node)| map_statement(&label_map, i, stmt_node))
            .collect();
        Compiler(CompiledProgram { instructions })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Jump, Operator, RegisterRef};

    #[test]
    fn test_delabel() {
        // Dummy span for comparisons
        let span = Span {
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
                Statement::Jump(
                    Node(Jump::Jmp, span),
                    Node("START".into(), span),
                ),
                span,
            ),
            Node(
                Statement::Operator(Node(
                    Operator::Read(Node(RegisterRef::User(0), span)),
                    span,
                )),
                span,
            ),
            Node(
                Statement::Jump(
                    Node(Jump::Jmp, span),
                    Node("START".into(), span),
                ),
                span,
            ),
            Node(
                Statement::Jump(
                    Node(Jump::Jmp, span),
                    Node("END".into(), span),
                ),
                span,
            ),
            Node(
                Statement::Operator(Node(
                    Operator::Read(Node(RegisterRef::User(0), span)),
                    span,
                )),
                span,
            ),
            Node(Statement::Label(Node(LabelDecl("END".into()), span)), span),
        ];
        let compiler = Compiler(SourceProgram { body });
        assert_eq!(
            compiler.delabel().0.instructions,
            vec![
                Node(Instruction::Jump(Node(Jump::Jmp, span), 0), span),
                Node(
                    Instruction::Operator(Node(
                        Operator::Read(Node(RegisterRef::User(0), span)),
                        span
                    )),
                    span
                ),
                Node(Instruction::Jump(Node(Jump::Jmp, span), -2), span),
                Node(Instruction::Jump(Node(Jump::Jmp, span), 2,), span),
                Node(
                    Instruction::Operator(Node(
                        Operator::Read(Node(RegisterRef::User(0,), span)),
                        span
                    )),
                    span
                ),
            ]
        );
    }
}
