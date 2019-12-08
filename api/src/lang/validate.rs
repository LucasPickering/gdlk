use crate::{
    error::{CompileError, CompileErrors},
    lang::{
        ast::{
            Instr, Operator, Program, RegisterRef, StackIdentifier, ValueSource,
        },
        Compiler,
    },
    models::Environment,
};

/// Helper method to change if a stack reference is in range. This is used for
/// mutliple error types so the comparison logic is pulled out here.
fn is_stack_ref_valid(env: &Environment, stack_id: StackIdentifier) -> bool {
    stack_id < (env.num_stacks as usize)
}

/// Ensures the register reference refers to a real register in the environment.
fn validate_reg_ref(env: &Environment, reg: &RegisterRef) -> CompileErrors {
    match reg {
        RegisterRef::InputLength => CompileErrors::none(),
        RegisterRef::StackLength(stack_id) => {
            if is_stack_ref_valid(env, *stack_id) {
                CompileErrors::none()
            } else {
                CompileError::InvalidRegisterRef(RegisterRef::StackLength(
                    *stack_id,
                ))
                .into()
            }
        }
        RegisterRef::User(reg_id) => {
            // TODO clean up this num conversion
            if *reg_id >= (env.num_registers as usize) {
                CompileError::InvalidRegisterRef(RegisterRef::User(*reg_id))
                    .into()
            } else {
                CompileErrors::none()
            }
        }
    }
}

/// Ensures the register reference refers to a real register in the environment.
fn validate_writeable_reg_ref(
    env: &Environment,
    reg: &RegisterRef,
) -> CompileErrors {
    // Make sure the reference points to a real register
    validate_reg_ref(env, reg)?;

    // If the reference is valid, we want to make sure it's writable too.
    // Only User registers are writable, all others cause an error.
    match reg {
        RegisterRef::User(_) => CompileErrors::none(),
        _ => CompileError::UnwritableRegister(*reg).into(),
    }
}

/// Ensures the stack ID refers to a real stack in the environment, i.e. makes
/// sure it's in bounds.
fn validate_stack_ref(
    env: &Environment,
    stack_id: StackIdentifier,
) -> CompileErrors {
    // TODO clean up this num conversion
    if is_stack_ref_valid(env, stack_id) {
        CompileErrors::none()
    } else {
        CompileError::InvalidStackRef(stack_id).into()
    }
}

/// Ensures the given ValueSource is valid. All constants are valid, but
/// register references need to be validated to make sure they refer to real
/// registers.
fn validate_val_src(env: &Environment, val_src: &ValueSource) -> CompileErrors {
    match val_src {
        ValueSource::Const(_) => CompileErrors::none(),
        ValueSource::Register(reg) => validate_reg_ref(env, reg),
    }
}

/// Collects all the validation errors in the given instruction. All possible
/// static validation is applied (mainly stack and register references).
fn validate_instr(env: &Environment, instr: &Instr) -> CompileErrors {
    match instr {
        Instr::Operator(op) => match op {
            Operator::Read(reg) | Operator::Write(reg) => {
                validate_writeable_reg_ref(env, reg)
            }
            Operator::Set(reg, val_src)
            | Operator::Add(reg, val_src)
            | Operator::Sub(reg, val_src)
            | Operator::Mul(reg, val_src) => {
                // Make sure the first reg is valid and writable, and the second
                // is a valid
                validate_writeable_reg_ref(env, reg)
                    .chain(validate_val_src(env, val_src))
            }
            Operator::Push(val_src, stack_id) => validate_val_src(env, val_src)
                .chain(validate_stack_ref(env, *stack_id)),
            Operator::Pop(stack_id, reg) => validate_stack_ref(env, *stack_id)
                .chain(validate_reg_ref(env, reg)),
        },
        Instr::If(reg, body) | Instr::While(reg, body) => {
            validate_reg_ref(env, reg).chain(validate_body(env, body))
        }
    }
}

/// Collects all the validation errors in all the instructions in the body.
fn validate_body(env: &Environment, body: &[Instr]) -> CompileErrors {
    body.iter().fold(CompileErrors::none(), |acc, instr| {
        acc.chain(validate_instr(env, instr))
    })
}

impl Compiler<Program> {
    /// Performs all possible static validation on the program. The environment
    /// is needed to determine what values and references are valid. If any
    /// errors occur, `Err` will be returned with all the errors in a
    /// collection.
    pub fn validate(
        self,
        env: &Environment,
    ) -> Result<Compiler<Program>, CompileErrors> {
        validate_body(env, &self.0.body)?;
        Ok(Compiler(self.0))
    }
}
