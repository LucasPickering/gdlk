use crate::{
    ast::{
        source::{LabelDecl, Program, Statement},
        Jump, Label, Node, Operator, RegisterRef, SpanNode, StackId, StackRef,
        ValueSource,
    },
    error::{CompileError, SourceErrorWrapper, WithSource},
    models::HardwareSpec,
    util::Span,
    Compiler,
};
use std::collections::HashMap;

struct Context<'a> {
    hardware_spec: &'a HardwareSpec,
    labels: &'a HashMap<&'a Label, Span>,
}

trait Validate {
    /// Validate a single object
    fn validate(
        &self,
        context: &Context,
        errors: &mut Vec<(CompileError, Span)>,
    );
}

impl Validate for SpanNode<Label> {
    fn validate(
        &self,
        context: &Context,
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
        context: &Context,
        errors: &mut Vec<(CompileError, Span)>,
    ) {
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
        context: &Context,
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
        context: &Context,
        errors: &mut Vec<(CompileError, Span)>,
    ) {
        if !is_stack_id_valid(context.hardware_spec, self.value().0) {
            errors.push((CompileError::InvalidStackRef, *self.metadata()))
        }
    }
}

impl Validate for SpanNode<Operator<Span>> {
    fn validate(
        &self,
        context: &Context,
        errors: &mut Vec<(CompileError, Span)>,
    ) {
        match self.value() {
            Operator::Read(reg_ref) => {
                reg_ref.validate(context, errors);
                validate_writable(errors, reg_ref);
            }
            Operator::Write(val_src) => val_src.validate(context, errors),
            Operator::Set(reg_ref, val_src)
            | Operator::Add(reg_ref, val_src)
            | Operator::Sub(reg_ref, val_src)
            | Operator::Mul(reg_ref, val_src) => {
                // Make sure the first reg is valid and writable, and the
                // second is a valid value source
                reg_ref.validate(context, errors);
                validate_writable(errors, reg_ref);
                val_src.validate(context, errors);
            }
            Operator::Cmp(reg_ref, val_src_1, val_src_2) => {
                reg_ref.validate(context, errors);
                validate_writable(errors, reg_ref);
                val_src_1.validate(context, errors);
                val_src_2.validate(context, errors);
            }
            Operator::Push(val_src, stack_ref) => {
                val_src.validate(context, errors);
                stack_ref.validate(context, errors);
            }
            Operator::Pop(stack_ref, reg_ref) => {
                stack_ref.validate(context, errors);
                reg_ref.validate(context, errors);
            }
        }
    }
}

impl Validate for SpanNode<Jump<Span>> {
    fn validate(
        &self,
        context: &Context,
        errors: &mut Vec<(CompileError, Span)>,
    ) {
        match self.value() {
            Jump::Jmp => {}
            Jump::Jez(val_src)
            | Jump::Jnz(val_src)
            | Jump::Jlz(val_src)
            | Jump::Jgz(val_src) => val_src.validate(context, errors),
        }
    }
}

impl Validate for SpanNode<Statement<Span>> {
    fn validate(
        &self,
        context: &Context,
        errors: &mut Vec<(CompileError, Span)>,
    ) {
        match self.value() {
            Statement::Label(_) => {}
            Statement::Operator(op) => op.validate(context, errors),
            Statement::Jump(jump, label) => {
                jump.validate(context, errors);
                label.validate(context, errors);
            }
        }
    }
}

/// Helper method to change if a stack reference is in range. This is used for
/// mutliple error types so the comparison logic is pulled out here.
fn is_stack_id_valid(hardware_spec: &HardwareSpec, stack_id: StackId) -> bool {
    stack_id < hardware_spec.num_stacks
}

/// Ensures the register reference refers to a writable register.
fn validate_writable(
    errors: &mut Vec<(CompileError, Span)>,
    reg_ref_node: &SpanNode<RegisterRef>,
) {
    // Only User registers are writable, all others cause an error.
    match reg_ref_node {
        Node(RegisterRef::User(_), _) => {}
        Node(_, span) => errors.push((CompileError::UnwritableRegister, *span)),
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
    hardware_spec: &HardwareSpec,
    body: &[SpanNode<Statement<Span>>],
) -> Vec<(CompileError, Span)> {
    let mut errors = Vec::new();
    let labels = collect_labels(&mut errors, body);
    let context = Context {
        hardware_spec,
        labels: &labels,
    };

    // Add in errors for each statement
    for stmt in body.iter() {
        stmt.validate(&context, &mut errors);
    }

    errors
}

impl Compiler<Program<Span>> {
    /// Performs all possible static validation on the program. The
    /// hardware is needed to determine what values and references
    /// are valid. If any errors occur, `Err` will be returned with all the
    /// errors in a collection.
    pub(crate) fn validate(
        self,
    ) -> Result<Compiler<Program<Span>>, WithSource<CompileError>> {
        let errors = validate_body(self.hardware_spec.inner(), &self.ast.body);
        if errors.is_empty() {
            Ok(self)
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
