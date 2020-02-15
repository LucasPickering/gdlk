use crate::{
    ast::{
        source::{Program, Statement},
        Jump, Label, Operator, RegisterRef, StackIdentifier, ValueSource,
    },
    error::{CompileError, CompileErrors},
    models::HardwareSpec,
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

impl Validate for Label {
    fn validate(&self, context: &Context) -> CompileErrors {
        if context.labels.contains(self) {
            CompileErrors::none()
        } else {
            CompileError::InvalidLabel(self.clone()).into()
        }
    }
}

impl Validate for RegisterRef {
    /// Ensures the register reference refers to a real register in the
    /// hardware.
    fn validate(&self, context: &Context) -> CompileErrors {
        match self {
            RegisterRef::InputLength => CompileErrors::none(),
            RegisterRef::StackLength(stack_id) => {
                if is_stack_ref_valid(context.hardware_spec, *stack_id) {
                    CompileErrors::none()
                } else {
                    CompileError::InvalidRegisterRef(RegisterRef::StackLength(
                        *stack_id,
                    ))
                    .into()
                }
            }
            RegisterRef::User(reg_id) => {
                if *reg_id >= context.hardware_spec.num_registers {
                    CompileError::InvalidRegisterRef(RegisterRef::User(*reg_id))
                        .into()
                } else {
                    CompileErrors::none()
                }
            }
        }
    }
}

impl Validate for ValueSource {
    /// Ensures the given ValueSource is valid. All constants are valid, but
    /// register references need to be validated to make sure they refer to real
    /// registers.
    fn validate(&self, context: &Context) -> CompileErrors {
        match self {
            ValueSource::Const(_) => CompileErrors::none(),
            ValueSource::Register(reg) => reg.validate(context),
        }
    }
}

impl Validate for StackIdentifier {
    /// Ensures the stack ID refers to a real stack in the hardware, i.e.
    /// makes sure it's in bounds.
    fn validate(&self, context: &Context) -> CompileErrors {
        if is_stack_ref_valid(context.hardware_spec, *self) {
            CompileErrors::none()
        } else {
            CompileError::InvalidStackRef(*self).into()
        }
    }
}

impl Validate for Operator {
    fn validate(&self, context: &Context) -> CompileErrors {
        match self {
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

impl Validate for Jump {
    fn validate(&self, context: &Context) -> CompileErrors {
        match self {
            Jump::Jmp => CompileErrors::none(),
            Jump::Jez(val_src)
            | Jump::Jnz(val_src)
            | Jump::Jlz(val_src)
            | Jump::Jgz(val_src) => val_src.validate(context),
        }
    }
}

impl Validate for Statement {
    fn validate(&self, context: &Context) -> CompileErrors {
        match self {
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
fn is_stack_ref_valid(
    hardware_spec: &HardwareSpec,
    stack_id: StackIdentifier,
) -> bool {
    stack_id < hardware_spec.num_stacks
}

/// Ensures the register reference refers to a writable register.
fn validate_writable(reg: &RegisterRef) -> CompileErrors {
    // Only User registers are writable, all others cause an error.
    match reg {
        RegisterRef::User(_) => CompileErrors::none(),
        _ => CompileError::UnwritableRegister(*reg).into(),
    }
}

/// Collect all labels in the program into a set. Returns errors for any
/// duplicate labels.
fn collect_labels(body: &[Statement]) -> (HashSet<&Label>, CompileErrors) {
    let mut labels = HashSet::new();
    let mut errors = CompileErrors::none();
    for stmt in body {
        if let Statement::Label(label) = stmt {
            // insert returns false if the value was already present
            if !labels.insert(label) {
                errors.push(CompileError::DuplicateLabel(label.clone()));
            }
        }
    }
    (labels, errors)
}

/// Collects all the validation errors in all the instructions in the body.
fn validate_body(
    hardware_spec: &HardwareSpec,
    body: &[Statement],
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

impl Compiler<Program> {
    /// Performs all possible static validation on the program. The
    /// hardware is needed to determine what values and references
    /// are valid. If any errors occur, `Err` will be returned with all the
    /// errors in a collection.
    pub fn validate(
        self,
        hardware_spec: &HardwareSpec,
    ) -> Result<Compiler<Program>, CompileErrors> {
        validate_body(hardware_spec, &self.0.body)?;
        Ok(Compiler(self.0))
    }
}
