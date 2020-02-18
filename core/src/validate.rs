use crate::{
    ast::{
        source::{Program, Statement},
        Jump, Label, Node, Operator, RegisterRef, Span, SpanNode, StackId,
        StackRef, ValueSource,
    },
    error::{CompileError, CompileErrors},
    models::HardwareSpec,
    util::Valid,
    Compiler,
};
use std::collections::HashSet;

struct Context<'a> {
    hardware_spec: &'a HardwareSpec,
    labels: &'a HashSet<&'a Label>,
}

trait Validate {
    /// Validate a single object
    fn validate(&self, context: &Context) -> CompileErrors;
}

impl Validate for SpanNode<Label> {
    fn validate(&self, context: &Context) -> CompileErrors {
        if context.labels.contains(&self.value()) {
            CompileErrors::none()
        } else {
            CompileError::InvalidLabel(self.clone()).into()
        }
    }
}

impl Validate for SpanNode<RegisterRef> {
    /// Ensures the register reference refers to a real register in the
    /// hardware.
    fn validate(&self, context: &Context) -> CompileErrors {
        match self.value() {
            RegisterRef::InputLength => CompileErrors::none(),
            RegisterRef::StackLength(stack_ref) => {
                if is_stack_id_valid(context.hardware_spec, *stack_ref) {
                    CompileErrors::none()
                } else {
                    CompileError::InvalidRegisterRef(*self).into()
                }
            }
            RegisterRef::User(reg_ref) => {
                if *reg_ref < context.hardware_spec.num_registers {
                    CompileErrors::none()
                } else {
                    CompileError::InvalidRegisterRef(*self).into()
                }
            }
        }
    }
}

impl Validate for SpanNode<ValueSource<Span>> {
    /// Ensures the given ValueSource is valid. All constants are valid, but
    /// register references need to be validated to make sure they refer to real
    /// registers.
    fn validate(&self, context: &Context) -> CompileErrors {
        match self.value() {
            ValueSource::Const(_) => CompileErrors::none(),
            ValueSource::Register(reg) => reg.validate(context),
        }
    }
}

impl Validate for SpanNode<StackRef> {
    /// Ensures the stack ID refers to a real stack in the hardware, i.e.
    /// makes sure it's in bounds.
    fn validate(&self, context: &Context) -> CompileErrors {
        if is_stack_id_valid(context.hardware_spec, self.value().0) {
            CompileErrors::none()
        } else {
            CompileError::InvalidStackRef(*self).into()
        }
    }
}

impl Validate for SpanNode<Operator<Span>> {
    fn validate(&self, context: &Context) -> CompileErrors {
        match self.value() {
            Operator::Read(reg_ref) => {
                reg_ref.validate(context).chain(validate_writable(reg_ref))
            }
            Operator::Write(val_src) => val_src.validate(context),
            Operator::Set(reg_ref, val_src)
            | Operator::Add(reg_ref, val_src)
            | Operator::Sub(reg_ref, val_src)
            | Operator::Mul(reg_ref, val_src) => {
                // Make sure the first reg is valid and writable, and the
                // second is a valid value source
                reg_ref
                    .validate(context)
                    .chain(validate_writable(reg_ref))
                    .chain(val_src.validate(context))
            }
            Operator::Cmp(reg_ref, val_src_1, val_src_2) => reg_ref
                .validate(context)
                .chain(validate_writable(reg_ref))
                .chain(val_src_1.validate(context))
                .chain(val_src_2.validate(context)),
            Operator::Push(val_src, stack_ref) => {
                val_src.validate(context).chain(stack_ref.validate(context))
            }
            Operator::Pop(stack_ref, reg_ref) => {
                stack_ref.validate(context).chain(reg_ref.validate(context))
            }
        }
    }
}

impl Validate for SpanNode<Jump<Span>> {
    fn validate(&self, context: &Context) -> CompileErrors {
        match self.value() {
            Jump::Jmp => CompileErrors::none(),
            Jump::Jez(val_src)
            | Jump::Jnz(val_src)
            | Jump::Jlz(val_src)
            | Jump::Jgz(val_src) => val_src.validate(context),
        }
    }
}

impl Validate for SpanNode<Statement<Span>> {
    fn validate(&self, context: &Context) -> CompileErrors {
        match self.value() {
            Statement::Label(_) => CompileErrors::none(),
            Statement::Operator(op) => op.validate(context),
            Statement::Jump(jump, label) => {
                jump.validate(context).chain(label.validate(context))
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
fn validate_writable(reg_ref: &SpanNode<RegisterRef>) -> CompileErrors {
    // Only User registers are writable, all others cause an error.
    match reg_ref.value() {
        RegisterRef::User(_) => CompileErrors::none(),
        _ => CompileError::UnwritableRegister(*reg_ref).into(),
    }
}

/// Collect all labels in the program into a set. Returns errors for any
/// duplicate labels.
fn collect_labels<'a>(
    body: &'a [SpanNode<Statement<Span>>],
) -> (HashSet<&'a Label>, CompileErrors) {
    let mut labels = HashSet::new();
    let mut errors = CompileErrors::none();
    for stmt in body {
        if let Node(Statement::Label(label_node), _) = stmt {
            // insert returns false if the value was already present
            if !labels.insert(&label_node.value().0) {
                errors.push(CompileError::DuplicateLabel(label_node.clone()));
            }
        }
    }
    (labels, errors)
}

/// Collects all the validation errors in all the instructions in the body.
fn validate_body(
    hardware_spec: &HardwareSpec,
    body: &[SpanNode<Statement<Span>>],
) -> CompileErrors {
    let (labels, errors) = collect_labels(body);
    let context = Context {
        hardware_spec,
        labels: &labels,
    };

    // Add in errors for each statement
    body.iter()
        .fold(errors, |acc, stmt| acc.chain(stmt.validate(&context)))
}

impl Compiler<Program<Span>> {
    /// Performs all possible static validation on the program. The
    /// hardware is needed to determine what values and references
    /// are valid. If any errors occur, `Err` will be returned with all the
    /// errors in a collection.
    pub fn validate(
        self,
        hardware_spec: &Valid<HardwareSpec>,
    ) -> Result<Compiler<Program<Span>>, CompileErrors> {
        validate_body(hardware_spec.inner(), &self.0.body)?;
        Ok(Compiler(self.0))
    }
}
