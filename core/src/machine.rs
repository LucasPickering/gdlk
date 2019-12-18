use crate::{
    ast::{
        LangValue, MachineInstr, Operator, RegisterRef, StackIdentifier,
        ValueSource,
    },
    consts::MAX_CYCLE_COUNT,
    debug,
    error::RuntimeError,
    models::{HardwareSpec, ProgramSpec},
};
use serde::Serialize;
use std::{
    cmp::Ordering, collections::VecDeque, convert::TryInto, iter,
    iter::FromIterator, num::Wrapping,
};

/// A steppable program executor. Maintains the current state of the program,
/// and execution can be progressed one instruction at a time.
///
/// Created from a [HardwareSpec](HardwareSpec), [ProgramSpec](ProgramSpec), and
/// a program. The current machine state can be obtained at any time, including
/// execution stats (e.g. # cycles), which allows for handy visualizations of
/// execution.
#[derive(Debug, PartialEq, Serialize)]
pub struct Machine {
    // Static data - this is copied from the input and shouldn't be included in
    // serialization. We store these ourselves instead of keeping references
    // to the originals because it just makes life a lot easier.
    #[serde(skip)]
    program: Vec<MachineInstr>,
    #[serde(skip)]
    expected_output: Vec<LangValue>,
    #[serde(skip)]
    max_stack_length: usize,

    // Runtime state
    /// The index of the next instruction to be executed
    program_counter: usize,
    /// The current input buffer. This can be popped from as the program is
    /// executed. The front of the buffer is where we want to pop from first,
    /// so this is a VecDeque so we get pop_front. Values can never be added to
    /// the input, only popped off.
    pub input: VecDeque<LangValue>,
    /// The current output buffer. This can be pushed into, but never popped
    /// out of.
    pub output: Vec<LangValue>,
    /// The registers that the user can read and write. Indexed by Register ID.
    pub registers: Vec<LangValue>,
    /// The series of stacks that act as the programs RAM. The number of stacks
    /// and their capacity is determined by the initializating hardware spec.
    pub stacks: Vec<Vec<LangValue>>,
    /// The number of instructions that have been executed so far. This is not
    /// unique, so repeated instructions are counted multiple times.
    pub cycle_count: usize,
}

impl Machine {
    /// Creates a new machine, ready to be executed.
    pub fn new(
        hardware_spec: &HardwareSpec,
        program_spec: &ProgramSpec,
        program: Vec<MachineInstr>,
    ) -> Self {
        Self {
            // Static data
            program,
            expected_output: program_spec.expected_output.clone(),
            max_stack_length: hardware_spec.max_stack_length,

            // Runtime state
            program_counter: 0,
            input: VecDeque::from_iter(program_spec.input.iter().copied()),
            output: Vec::new(),
            registers: iter::repeat(0)
                .take(hardware_spec.num_registers)
                .collect(),
            // Initialize `num_stacks` new stacks. Set an initial capacity
            // for each one to prevent grows during program operation
            stacks: iter::repeat_with(|| {
                Vec::with_capacity(hardware_spec.max_stack_length)
            })
            .take(hardware_spec.num_stacks)
            .collect(),

            // Performance stats
            cycle_count: 0,
        }
    }

    /// Gets a source value, which could either be a constant or a register.
    /// If the value is a constant, just return that. If it's a register,
    /// return the value from that register. Returns `RuntimeError` if it
    /// is an invalid register reference.
    fn get_value_from_source(&self, src: &ValueSource) -> LangValue {
        match src {
            ValueSource::Const(val) => *val,
            ValueSource::Register(reg) => self.get_reg(reg),
        }
    }

    /// Gets the value from the given register. The register reference is
    /// assumed to be valid (should be validated at build time). Will panic if
    /// it isn't valid.
    fn get_reg(&self, reg: &RegisterRef) -> LangValue {
        match reg {
            // These conversion unwraps are safe because we know that input
            // and stack lengths are bounded by validation rules to fit into an
            // i32 (max length is 256 at the time of writing this)
            RegisterRef::InputLength => self.input.len().try_into().unwrap(),
            RegisterRef::StackLength(stack_id) => {
                self.stacks[*stack_id].len().try_into().unwrap()
            }
            RegisterRef::User(reg_id) => *self.registers.get(*reg_id).unwrap(),
        }
    }

    /// Sets the register to the given value. The register reference is
    /// assumed to be valid and writable (should be validated at build time).
    /// Will panic if it isn't valid/writable.
    fn set_reg(&mut self, reg: &RegisterRef, value: LangValue) {
        match reg {
            RegisterRef::User(reg_id) => {
                self.registers[*reg_id] = value;
            }
            _ => panic!("Unwritable register {}", reg),
        }
    }

    /// Pushes the given value onto the given stack. If the stack reference is
    /// invalid or the stack is at capacity, an error is returned. If the stack
    /// reference is invalid, will panic (should be validated at build time).
    fn push_stack(
        &mut self,
        stack_id: StackIdentifier,
        value: LangValue,
    ) -> Result<(), RuntimeError> {
        // Have to access this first cause borrow checker
        let max_stack_length = self.max_stack_length;
        let stack = &mut self.stacks[stack_id];

        // If the stack is capacity, make sure we're not over it
        if stack.len() >= max_stack_length {
            return Err(RuntimeError::StackOverflow(stack_id));
        }

        stack.push(value);
        Ok(())
    }

    /// Pops an element off the given stack. If the pop is successful, the
    /// popped value is returned. If the stack is empty, an error is returned.
    /// If the stack reference is invalid, will panic (should be validated at
    /// build time).
    fn pop_stack(
        &mut self,
        stack_id: StackIdentifier,
    ) -> Result<LangValue, RuntimeError> {
        let stack = &mut self.stacks[stack_id];

        if let Some(val) = stack.pop() {
            Ok(val)
        } else {
            Err(RuntimeError::EmptyStack(stack_id))
        }
    }

    /// Executes the next instruction in the program. If there are no
    /// instructions left to execute, this panics.
    pub fn execute_next(&mut self) -> Result<(), RuntimeError> {
        // Prevent infinite loops
        if self.cycle_count >= MAX_CYCLE_COUNT {
            return Err(RuntimeError::TooManyCycles);
        }

        let instr = *self
            .program
            .get(self.program_counter)
            .ok_or(RuntimeError::ProgramTerminated)?;

        // Execute the instruction. For most instructions, the number of
        // instructions to consume is just 1, so for those return None. For
        // jumps though, it can vary so they'll return Some(n).
        let instrs_to_consume: Option<i32> = match instr {
            MachineInstr::Operator(op) => {
                match op {
                    Operator::Read(reg) => match self.input.pop_front() {
                        Some(val) => {
                            self.set_reg(&reg, val);
                        }
                        None => return Err(RuntimeError::EmptyInput),
                    },
                    Operator::Write(reg) => {
                        self.output.push(self.get_reg(&reg));
                    }
                    Operator::Set(dst, src) => {
                        self.set_reg(&dst, self.get_value_from_source(&src));
                    }
                    Operator::Add(dst, src) => {
                        self.set_reg(
                            &dst,
                            (Wrapping(self.get_reg(&dst))
                                + Wrapping(self.get_value_from_source(&src)))
                            .0,
                        );
                    }
                    Operator::Sub(dst, src) => {
                        self.set_reg(
                            &dst,
                            (Wrapping(self.get_reg(&dst))
                                - Wrapping(self.get_value_from_source(&src)))
                            .0,
                        );
                    }
                    Operator::Mul(dst, src) => {
                        self.set_reg(
                            &dst,
                            (Wrapping(self.get_reg(&dst))
                                * Wrapping(self.get_value_from_source(&src)))
                            .0,
                        );
                    }
                    Operator::Cmp(dst, src_1, src_2) => {
                        let val_1 = self.get_value_from_source(&src_1);
                        let val_2 = self.get_value_from_source(&src_2);
                        let cmp = match val_1.cmp(&val_2) {
                            Ordering::Less => -1,
                            Ordering::Equal => 0,
                            Ordering::Greater => 1,
                        };
                        self.set_reg(&dst, cmp);
                    }
                    Operator::Push(src, stack_id) => {
                        self.push_stack(
                            stack_id,
                            self.get_value_from_source(&src),
                        )?;
                    }
                    Operator::Pop(stack_id, dst) => {
                        let popped = self.pop_stack(stack_id)?;
                        self.set_reg(&dst, popped);
                    }
                }
                None
            }

            MachineInstr::Jez(num_instrs, reg) => {
                if self.get_reg(&reg) == 0 {
                    Some(num_instrs)
                } else {
                    None
                }
            }
            MachineInstr::Jnz(num_instrs, reg) => {
                if self.get_reg(&reg) != 0 {
                    Some(num_instrs)
                } else {
                    None
                }
            }
        };

        // Advance the pc by the specified number of instructions (for jumps)
        // or by 1 (for all other instructions). Overflow/underflow _shouldn't_
        // occur here, but if it does, that should panic in debug mode and
        // cause all kinds of fuckery in release mode.
        self.program_counter = (self.program_counter as i32
            + instrs_to_consume.unwrap_or(1))
            as usize;
        self.cycle_count += 1;
        debug!(println!("Executed {:?}\n\tState: {:?}", instr, self));
        Ok(())
    }

    /// Executes this machine until termination (or error). All instructions are
    /// executed until [is_complete](Machine::is_complete) returns true. Returns
    /// the value of [is_successful](Machine::is_successful) upon termination.
    pub fn execute_all(&mut self) -> Result<bool, RuntimeError> {
        while !self.is_complete() {
            self.execute_next()?;
        }
        Ok(self.is_successful())
    }

    /// Checks if this machine has finished executing.
    pub fn is_complete(&self) -> bool {
        self.program_counter >= self.program.len()
    }

    /// Checks if this machine has completed successfully. The criteria are:
    /// 1. Program is complete (all instructions have been executed)
    /// 2. Input buffer has been exhausted (all input has been consumed)
    /// 3. Output buffer matches the expected output, as defined by the
    /// [ProgramSpec](ProgramSpec)
    pub fn is_successful(&self) -> bool {
        self.is_complete()
            && self.input.is_empty()
            && self.output == self.expected_output
    }
}
