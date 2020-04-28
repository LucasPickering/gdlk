use crate::{
    ast::{
        compiled::{Instruction, Program},
        Jump, LangValue, Node, Operator, RegisterRef, SpanNode, StackRef,
        ValueSource,
    },
    consts::MAX_CYCLE_COUNT,
    debug,
    error::{RuntimeError, SourceErrorWrapper, WithSource},
    models::{HardwareSpec, ProgramSpec},
    util::{Span, Valid},
};
use std::{
    cmp::Ordering, collections::HashMap, convert::TryInto, iter, num::Wrapping,
};

/// A steppable program executor. Maintains the current state of the program,
/// and execution can be progressed one instruction at a time.
///
/// Created from a [HardwareSpec](HardwareSpec), [ProgramSpec](ProgramSpec), and
/// a program. The current machine state can be obtained at any time, including
/// execution stats (e.g. # cycles), which allows for handy visualizations of
/// execution.
#[derive(Debug)]
pub struct Machine {
    // Static data - this is copied from the input and shouldn't be included in
    // serialization. We store these ourselves instead of keeping references
    // to the originals because it just makes life a lot easier.
    hardware_spec: Valid<HardwareSpec>,
    source: String,
    program: Program<Span>,
    expected_output: Vec<LangValue>,

    // Runtime state
    /// The index of the next instruction to be executed
    program_counter: usize,
    /// The current input buffer. This can be popped from as the program is
    /// executed. We always pop from the front. This isn't ideal for a Vec,
    /// but these arrays will be small enough that it probably doesn't matter.
    /// Values never get added to the input, only popped off.
    pub input: Vec<LangValue>,
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
        hardware_spec: &Valid<HardwareSpec>,
        program_spec: &Valid<ProgramSpec>,
        program: Program<Span>,
        source: String,
    ) -> Self {
        let registers =
            iter::repeat(0).take(hardware_spec.num_registers).collect();

        // Initialize `num_stacks` new stacks. Set an initial capacity
        // for each one to prevent grows during program operation
        let stacks = iter::repeat_with(|| {
            Vec::with_capacity(hardware_spec.max_stack_length)
        })
        .take(hardware_spec.num_stacks)
        .collect();

        Self {
            // Static data
            hardware_spec: *hardware_spec,
            program,
            source,
            expected_output: program_spec.expected_output.clone(),

            // Runtime state
            program_counter: 0,
            input: program_spec.input.clone(),
            output: Vec::new(),
            registers,
            stacks,

            // Performance stats
            cycle_count: 0,
        }
    }

    /// Gets a source value, which could either be a constant or a register.
    /// If the value is a constant, just return that. If it's a register,
    /// return the value from that register. Panics if the register reference is
    /// invalid (shouldn't be possible because of validation).
    fn get_val_from_src(&self, src: &SpanNode<ValueSource<Span>>) -> LangValue {
        match src.value() {
            ValueSource::Const(Node(val, _)) => *val,
            ValueSource::Register(reg_ref) => self.get_reg(*reg_ref.value()),
        }
    }

    /// Gets the value from the given register. The register reference is
    /// assumed to be valid (should be validated at build time). Will panic if
    /// it isn't valid.
    fn get_reg(&self, reg: RegisterRef) -> LangValue {
        match reg {
            // These conversion unwraps are safe because we know that input
            // and stack lengths are bounded by validation rules to fit into an
            // i32 (max length is 256 at the time of writing this)
            RegisterRef::InputLength => self.input.len().try_into().unwrap(),
            RegisterRef::StackLength(stack_id) => {
                self.stacks[stack_id].len().try_into().unwrap()
            }
            RegisterRef::User(reg_id) => *self.registers.get(reg_id).unwrap(),
        }
    }

    /// Sets the register to the given value. The register reference is
    /// assumed to be valid and writable (should be validated at build time).
    /// Will panic if it isn't valid/writable.
    fn set_reg(&mut self, reg: &SpanNode<RegisterRef>, value: LangValue) {
        match reg.value() {
            RegisterRef::User(reg_id) => {
                self.registers[*reg_id] = value;
            }
            _ => panic!("Unwritable register {:?}", reg),
        }
    }

    /// Pushes the given value onto the given stack. If the stack reference is
    /// invalid or the stack is at capacity, an error is returned. If the stack
    /// reference is invalid, will panic (should be validated at build time).
    fn push_stack(
        &mut self,
        stack_ref: &SpanNode<StackRef>,
        value: LangValue,
    ) -> Result<(), (RuntimeError, Span)> {
        // Have to access this first cause borrow checker
        let max_stack_length = self.hardware_spec.max_stack_length;
        let stack = &mut self.stacks[stack_ref.value().0];

        // If the stack is capacity, make sure we're not over it
        if stack.len() >= max_stack_length {
            return Err((RuntimeError::StackOverflow, *stack_ref.metadata()));
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
        stack_ref: &SpanNode<StackRef>,
    ) -> Result<LangValue, (RuntimeError, Span)> {
        let stack = &mut self.stacks[stack_ref.value().0];

        if let Some(val) = stack.pop() {
            Ok(val)
        } else {
            Err((RuntimeError::EmptyStack, *stack_ref.metadata()))
        }
    }

    /// Internal function to execute the next instruction. The return value
    /// is the same as [Self::execute_next], except the error needs to be
    /// wrapped before being handed to the user.
    fn execute_next_inner(&mut self) -> Result<bool, (RuntimeError, Span)> {
        let instr_node: SpanNode<Instruction<Span>> =
            match self.program.instructions.get(self.program_counter) {
                Some(instr_node) => *instr_node,
                // out of instructions to execute, just give up
                None => return Ok(false),
            };

        // Prevent infinite loops
        if self.cycle_count >= MAX_CYCLE_COUNT {
            // Include the instruction that triggered the error
            return Err((RuntimeError::TooManyCycles, *instr_node.metadata()));
        }

        // Execute the instruction. For most instructions, the number of
        // instructions to consume is just 1. For jumps though, it can vary.
        let instr = instr_node.value();
        let instrs_to_consume: isize = match instr {
            // Operators
            Instruction::Operator(Node(op, span)) => {
                match op {
                    Operator::Read(reg) => {
                        if self.input.is_empty() {
                            return Err((RuntimeError::EmptyInput, *span));
                        } else {
                            // Remove the first element in the input
                            let val = self.input.remove(0);
                            self.set_reg(&reg, val);
                        }
                    }
                    Operator::Write(src) => {
                        self.output.push(self.get_val_from_src(&src));
                    }
                    Operator::Set(dst, src) => {
                        self.set_reg(&dst, self.get_val_from_src(&src));
                    }
                    Operator::Add(dst, src) => {
                        self.set_reg(
                            &dst,
                            (Wrapping(self.get_reg(*dst.value()))
                                + Wrapping(self.get_val_from_src(&src)))
                            .0,
                        );
                    }
                    Operator::Sub(dst, src) => {
                        self.set_reg(
                            &dst,
                            (Wrapping(self.get_reg(*dst.value()))
                                - Wrapping(self.get_val_from_src(&src)))
                            .0,
                        );
                    }
                    Operator::Mul(dst, src) => {
                        self.set_reg(
                            &dst,
                            (Wrapping(self.get_reg(*dst.value()))
                                * Wrapping(self.get_val_from_src(&src)))
                            .0,
                        );
                    }
                    Operator::Cmp(dst, src_1, src_2) => {
                        let val_1 = self.get_val_from_src(&src_1);
                        let val_2 = self.get_val_from_src(&src_2);
                        let cmp = match val_1.cmp(&val_2) {
                            Ordering::Less => -1,
                            Ordering::Equal => 0,
                            Ordering::Greater => 1,
                        };
                        self.set_reg(&dst, cmp);
                    }
                    Operator::Push(src, stack_ref) => {
                        self.push_stack(
                            &stack_ref,
                            self.get_val_from_src(&src),
                        )?;
                    }
                    Operator::Pop(stack_ref, dst) => {
                        let popped = self.pop_stack(&stack_ref)?;
                        self.set_reg(&dst, popped);
                    }
                }
                1
            }

            // Jumps
            Instruction::Jump(Node(jump, _), offset) => {
                let should_jump = match jump {
                    Jump::Jmp => true,
                    Jump::Jez(src) => self.get_val_from_src(&src) == 0,
                    Jump::Jnz(src) => self.get_val_from_src(&src) != 0,
                    Jump::Jlz(src) => self.get_val_from_src(&src) < 0,
                    Jump::Jgz(src) => self.get_val_from_src(&src) > 0,
                };
                if should_jump {
                    *offset
                } else {
                    1
                }
            }
        };

        // Advance the pc by the specified number of instructions (for jumps)
        // or by 1 (for all other instructions). Overflow/underflow _shouldn't_
        // occur here, but if it does, that should panic in debug mode and
        // cause all kinds of fuckery in release mode.
        self.program_counter =
            (self.program_counter as isize + instrs_to_consume) as usize;
        self.cycle_count += 1;
        debug!(println!("Executed {:?}\n\tState: {:?}", instr, self));
        Ok(true)
    }

    /// Executes the next instruction in the program. Return value:
    /// - `Ok(true)`: One instruction was executed, machine state was updated
    /// - `Ok(false)`: No instructions left to execute, machine state was not
    ///   changed
    /// - `Err((error, span))`: An error occurred. The error and the span of the
    ///   source that caused it are returned
    pub fn execute_next(&mut self) -> Result<bool, WithSource<RuntimeError>> {
        self.execute_next_inner().map_err(|(error, span)| {
            WithSource::new(
                iter::once(SourceErrorWrapper::new(error, span, &self.source)),
                self.source.clone(),
            )
        })
    }

    /// Executes this machine until termination (or error). All instructions are
    /// executed until [is_complete](Machine::is_complete) returns true. Returns
    /// the value of [is_successful](Machine::is_successful) upon termination.
    pub fn execute_all(&mut self) -> Result<bool, WithSource<RuntimeError>> {
        while !self.is_complete() {
            self.execute_next()?;
        }
        Ok(self.is_successful())
    }

    /// Get the index of the next instruction to be executed.
    pub fn program_counter(&self) -> usize {
        self.program_counter
    }

    /// Get the current input buffer.
    pub fn input(&self) -> &[LangValue] {
        self.input.as_slice()
    }

    /// Get the current output buffer.
    pub fn output(&self) -> &[LangValue] {
        self.output.as_slice()
    }

    /// Get all registers and their current values.
    pub fn registers(&self) -> HashMap<RegisterRef, LangValue> {
        self.hardware_spec
            .all_register_refs()
            .into_iter()
            .map(|reg_ref| (reg_ref, self.get_reg(reg_ref)))
            .collect()
    }

    /// Get all stacks and their current values.
    pub fn stacks(&self) -> HashMap<StackRef, &[LangValue]> {
        self.hardware_spec
            .all_stack_refs()
            .into_iter()
            .map(|stack_ref| (stack_ref, self.stacks[stack_ref.0].as_slice()))
            .collect()
    }

    /// Checks if this machine has finished executing.
    pub fn is_complete(&self) -> bool {
        self.program_counter >= self.program.instructions.len()
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
