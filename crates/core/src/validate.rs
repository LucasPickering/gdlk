use crate::{
    ast::{
        source::{LabelDecl, Program, Statement},
        Instruction, Label, Node, RegisterRef, SpanNode, StackId, StackRef,
        ValueSource,
    },
    error::{CompileError, SourceErrorWrapper, WithSource},
    models::HardwareSpec,
    util::Span,
    Compiler, ProgramStats,
};
use std::collections::{HashMap, HashSet};

struct Context<'a> {
    hardware_spec: HardwareSpec,
    labels: HashMap<&'a Label, Span>,
    stats: ProgramStats,
}

impl<'a> Context<'a> {
    /// Add a register reference to the list of all references made in the
    /// program, for stat tracking purposes. This can be called even if an
    /// error is present, because in the error case we won't return the stats.
    fn add_register_ref(&mut self, register_ref: RegisterRef) {
        self.stats.referenced_registers.insert(register_ref);
    }

    /// Add a stack reference to the list of all references made in the
    /// program, for stat tracking purposes. This can be called even if an
    /// error is present, because in the error case we won't return the stats.
    fn add_stack_ref(&mut self, stack_ref: StackRef) {
        self.stats.referenced_stacks.insert(stack_ref);
    }
}

trait Validate {
    /// Validate a single object
    fn validate(
        &self,
        context: &mut Context,
        errors: &mut Vec<(CompileError, Span)>,
    );
}

impl Validate for SpanNode<Label> {
    fn validate(
        &self,
        context: &mut Context,
        errors: &mut Vec<(CompileError, Span)>,
    ) {
        if !context.labels.contains_key(&self.value()) {
            errors.push((CompileError::InvalidLabel, *self.metadata()))
        }
    }
}

impl Validate for SpanNode<RegisterRef> {
    /// Ensures the register reference refers to a real register in the
    /// hardware.
    fn validate(
        &self,
        context: &mut Context,
        errors: &mut Vec<(CompileError, Span)>,
    ) {
        // Track this reference in the stats
        context.add_register_ref(*self.value());
        match self {
            Node(RegisterRef::StackLength(stack_ref), span)
                if !is_stack_id_valid(context.hardware_spec, *stack_ref) =>
            {
                errors.push((CompileError::InvalidRegisterRef, *span))
            }
            Node(RegisterRef::User(reg_ref), span) => {
                if *reg_ref >= context.hardware_spec.num_registers {
                    errors.push((CompileError::InvalidRegisterRef, *span))
                }
            }
            _ => {}
        }
    }
}

impl Validate for SpanNode<ValueSource<Span>> {
    /// Ensures the given ValueSource is valid. All constants are valid, but
    /// register references need to be validated to make sure they refer to real
    /// registers.
    fn validate(
        &self,
        context: &mut Context,
        errors: &mut Vec<(CompileError, Span)>,
    ) {
        match self.value() {
            ValueSource::Const(_) => {}
            ValueSource::Register(reg) => reg.validate(context, errors),
        }
    }
}

impl Validate for SpanNode<StackRef> {
    /// Ensures the stack ID refers to a real stack in the hardware, i.e.
    /// makes sure it's in bounds.
    fn validate(
        &self,
        context: &mut Context,
        errors: &mut Vec<(CompileError, Span)>,
    ) {
        // Track this reference in the stats
        context.add_stack_ref(*self.value());
        if !is_stack_id_valid(context.hardware_spec, self.value().0) {
            errors.push((CompileError::InvalidStackRef, *self.metadata()))
        }
    }
}

impl Validate for SpanNode<Instruction<Span>> {
    fn validate(
        &self,
        context: &mut Context,
        errors: &mut Vec<(CompileError, Span)>,
    ) {
        match self.value() {
            Instruction::Read(reg_ref) => {
                reg_ref.validate(context, errors);
                validate_writable(errors, reg_ref);
            }
            Instruction::Write(val_src) => val_src.validate(context, errors),
            Instruction::Set(reg_ref, val_src)
            | Instruction::Add(reg_ref, val_src)
            | Instruction::Sub(reg_ref, val_src)
            | Instruction::Mul(reg_ref, val_src)
            | Instruction::Div(reg_ref, val_src) => {
                // Make sure the first reg is valid and writable, and the
                // second is a valid value source
                reg_ref.validate(context, errors);
                validate_writable(errors, reg_ref);
                val_src.validate(context, errors);
            }
            Instruction::Cmp(reg_ref, val_src_1, val_src_2) => {
                reg_ref.validate(context, errors);
                validate_writable(errors, reg_ref);
                val_src_1.validate(context, errors);
                val_src_2.validate(context, errors);
            }
            Instruction::Push(val_src, stack_ref) => {
                val_src.validate(context, errors);
                stack_ref.validate(context, errors);
            }
            Instruction::Pop(stack_ref, reg_ref) => {
                stack_ref.validate(context, errors);
                reg_ref.validate(context, errors);
            }

            // Jumps
            Instruction::Jmp(label) => {
                label.validate(context, errors);
            }
            Instruction::Jez(val_src, label)
            | Instruction::Jnz(val_src, label)
            | Instruction::Jlz(val_src, label)
            | Instruction::Jgz(val_src, label) => {
                val_src.validate(context, errors);
                label.validate(context, errors);
            }
        }
    }
}

impl Validate for SpanNode<Statement<Span>> {
    fn validate(
        &self,
        context: &mut Context,
        errors: &mut Vec<(CompileError, Span)>,
    ) {
        match self.value() {
            Statement::Label(_) => {}
            Statement::Instruction(instruction) => {
                instruction.validate(context, errors)
            }
        }
    }
}

/// Helper method to change if a stack reference is in range. This is used for
/// mutliple error types so the comparison logic is pulled out here.
fn is_stack_id_valid(hardware_spec: HardwareSpec, stack_id: StackId) -> bool {
    stack_id < hardware_spec.num_stacks
}

/// Ensures the register reference refers to a writable register.
fn validate_writable(
    errors: &mut Vec<(CompileError, Span)>,
    reg_ref_node: &SpanNode<RegisterRef>,
) {
    // Only User registers are writable, all others cause an error.
    match reg_ref_node {
        Node(RegisterRef::Null | RegisterRef::User(_), _) => {}
        Node(RegisterRef::InputLength | RegisterRef::StackLength(_), span) => {
            errors.push((CompileError::UnwritableRegister, *span))
        }
    }
}

/// Collect all labels in the program into a set. Returns errors for any
/// duplicate labels.
fn collect_labels<'a>(
    errors: &mut Vec<(CompileError, Span)>,
    body: &'a [SpanNode<Statement<Span>>],
) -> HashMap<&'a Label, Span> {
    let mut labels: HashMap<&'a Label, Span> = HashMap::new();
    for stmt in body {
        if let Node(Statement::Label(Node(LabelDecl(label), span)), _) = stmt {
            // insert returns false if the value was already present
            if let Some(original_span) = labels.get(&label) {
                errors.push((
                    CompileError::DuplicateLabel {
                        original: *original_span,
                    },
                    *span,
                ));
            } else {
                labels.insert(label, *span);
            }
        }
    }
    labels
}

/// Collects all the validation errors in all the instructions in the body.
fn validate_body(
    hardware_spec: HardwareSpec,
    body: &[SpanNode<Statement<Span>>],
) -> (ProgramStats, Vec<(CompileError, Span)>) {
    let mut errors = Vec::new();
    let labels = collect_labels(&mut errors, body);
    let mut context = Context {
        hardware_spec,
        labels,
        // This will be updated as we traverse the tree
        stats: ProgramStats {
            referenced_registers: HashSet::new(),
            referenced_stacks: HashSet::new(),
        },
    };

    // Add in errors for each statement
    for stmt in body.iter() {
        stmt.validate(&mut context, &mut errors);
    }

    (context.stats, errors)
}

impl Compiler<Program<Span>> {
    /// Performs all possible static validation on the program. The
    /// hardware is needed to determine what values and references
    /// are valid. If any errors occur, `Err` will be returned with all the
    /// errors in a collection.
    ///
    /// This step also collects static statistics on the program, such as
    /// which registers were referenced, which stats were referenced, etc. See
    /// [ProgramStats] for all the stats that are collected.
    pub(crate) fn validate(
        self,
    ) -> Result<Compiler<(Program<Span>, ProgramStats)>, WithSource<CompileError>>
    {
        let (stats, errors) = validate_body(self.hardware_spec, &self.ast.body);
        if errors.is_empty() {
            Ok(Compiler {
                source: self.source,
                hardware_spec: self.hardware_spec,
                ast: (self.ast, stats),
            })
        } else {
            let errors: Vec<_> = errors
                .into_iter()
                .map(|(error, span)| {
                    SourceErrorWrapper::new(error, span, &self.source)
                })
                .collect();
            Err(WithSource::new(errors, self.source))
        }
    }
}
