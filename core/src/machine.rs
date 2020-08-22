#[cfg(feature = "wasm")]
use crate::ast::wasm::{LangValueArrayMap, LangValueMap, SourceElement};
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
    util::Span,
};
use std::{
    cmp::Ordering, collections::HashMap, convert::TryInto, iter, num::Wrapping,
};
#[cfg(feature = "wasm")]
use wasm_bindgen::{prelude::*, JsCast};

/// A steppable program executor. Maintains the current state of the program,
/// and execution can be progressed one instruction at a time.
///
/// Created from a [HardwareSpec](HardwareSpec), [ProgramSpec](ProgramSpec), and
/// a program. The current machine state can be obtained at any time, including
/// execution stats (e.g. # cycles), which allows for handy visualizations of
/// execution.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Clone, Debug)]
pub struct Machine {
    // Static data - this is copied from the input and shouldn't be included in
    // serialization. We store these ourselves instead of keeping references
    // to the originals because it just makes life a lot easier.
    hardware_spec: HardwareSpec,
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
    input: Vec<LangValue>,
    /// The current output buffer. This can be pushed into, but never popped
    /// out of.
    output: Vec<LangValue>,
    /// The registers that the user can read and write. Indexed by Register ID.
    registers: Vec<LangValue>,
    /// The series of stacks that act as the programs RAM. The number of stacks
    /// and their capacity is determined by the initializating hardware spec.
    stacks: Vec<Vec<LangValue>>,
    /// The number of instructions that have been executed so far. This is not
    /// unique, so repeated instructions are counted multiple times.
    cycle_count: usize,
    /// Stores a runtime error, if one has occurred. Once the error occurs,
    /// this should be populated and from then on, the machine has terminated
    /// and can no longer execute.
    error: Option<WithSource<RuntimeError>>,
}

// Functions that DON'T get exported to wasm
impl Machine {
    /// Creates a new machine, ready to be executed.
    pub fn new(
        hardware_spec: HardwareSpec,
        program_spec: &ProgramSpec,
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
            hardware_spec,
            program,
            source,
            expected_output: program_spec.expected_output().into(),

            // Runtime state
            program_counter: 0,
            input: program_spec.input().into(),
            output: Vec::new(),
            registers,
            stacks,

            // Performance stats
            cycle_count: 0,
            error: None,
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
            RegisterRef::Null => 0,
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
            RegisterRef::Null => {} // /dev/null behavior - trash any input
            RegisterRef::InputLength | RegisterRef::StackLength(_) => {
                panic!("Unwritable register {:?}", reg)
            }
            RegisterRef::User(reg_id) => {
                self.registers[*reg_id] = value;
            }
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
        // We've previously hit an error, prevent further execution
        if self.error.is_some() {
            return Ok(false);
        }

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

        // If we've reached this point, we know we're going to execute the
        // instruction. Increment the cycle count now so that if we exit with
        // an error, it still counts.
        self.cycle_count += 1;

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
                    Operator::Div(dst, src) => {
                        let divisor = self.get_val_from_src(&src);
                        let dividend = self.get_reg(*dst.value());
                        if divisor != 0 {
                            // This does flooring division
                            self.set_reg(&dst, dividend / divisor);
                        } else {
                            return Err((RuntimeError::DivideByZero, *span));
                        }
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
        debug!(println!("Executed {:?}\n\tState: {:?}", instr, self));
        Ok(true)
    }

    /// Executes the next instruction in the program.
    ///
    /// # Returns
    /// - `Ok(true)` if the instruction executed normally
    /// - `Ok(false)` if the instruction didn't execute because the program has
    ///   already terminated
    /// - `Err(error)` if an error occurred. The error is returned, with the
    ///   source information of the offending instruction
    pub fn execute_next(&mut self) -> Result<bool, &WithSource<RuntimeError>> {
        match self.execute_next_inner() {
            Ok(b) => Ok(b),
            Err((error, span)) => {
                // Store the error in self, then return a ref to it
                self.error = Some(WithSource::new(
                    iter::once(SourceErrorWrapper::new(
                        error,
                        span,
                        &self.source,
                    )),
                    self.source.clone(),
                ));
                Err(self.error.as_ref().unwrap())
            }
        }
    }

    /// Executes this machine until termination (or error). All instructions are
    /// executed until [Self::terminated] returns true. Returns the value of
    /// [Self::successful] upon termination.
    pub fn execute_all(&mut self) -> Result<bool, &WithSource<RuntimeError>> {
        // We can't return the error directly from the loop because of a bug
        // in the borrow checker. Instead, we have to play lifetime tetris.
        while !self.terminated() {
            if self.execute_next().is_err() {
                break;
            }
        }

        // Check if an error occurred, and return it if so
        match &self.error {
            None => Ok(self.successful()),
            Some(error) => Err(error),
        }
    }

    /// Get a reference to the program being executed.
    pub fn program(&self) -> &Program<Span> {
        &self.program
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

    /// Get the runtime error that halted execution of this machine. If no error
    /// has occurred, return `None`.
    pub fn error(&self) -> Option<&WithSource<RuntimeError>> {
        self.error.as_ref()
    }
}

// Functions that get exported to wasm
#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl Machine {
    /// Get the index of the next instruction to be executed.
    #[cfg_attr(
        feature = "wasm",
        wasm_bindgen(getter, js_name = "programCounter")
    )]
    pub fn program_counter(&self) -> usize {
        self.program_counter
    }

    /// Get the number of cycles, i.e. the number of instructions that have
    /// been run, during the current program execution.
    #[cfg_attr(feature = "wasm", wasm_bindgen(getter, js_name = "cycleCount"))]
    pub fn cycle_count(&self) -> usize {
        self.cycle_count
    }

    /// Checks if this machine has finished executing. This could be by normal
    /// completion or by runtime error.
    #[cfg_attr(feature = "wasm", wasm_bindgen(getter, js_name = "terminated"))]
    pub fn terminated(&self) -> bool {
        // Check for normal complete
        self.program_counter >= self.program.instructions.len()
        // Check for a runtime error
            || self.error.is_some()
    }

    /// Checks if this machine has completed successfully. The criteria are:
    /// 1. Program is terminated (all instructions have been executed)
    /// 2. Program terminated normally (a runtime error did NOT occur)
    /// 3. Input buffer has been exhausted (all input has been consumed)
    /// 4. Output buffer matches the expected output, as defined by the
    /// [ProgramSpec](ProgramSpec)
    #[cfg_attr(feature = "wasm", wasm_bindgen(getter, js_name = "successful"))]
    pub fn successful(&self) -> bool {
        self.terminated()
            && self.error.is_none()
            && self.input.is_empty()
            && self.output == self.expected_output
    }

    /// A wrapper for [Self::input], to be called from wasm.
    #[cfg(feature = "wasm")]
    #[wasm_bindgen(getter, js_name = "input")]
    pub fn wasm_input(&self) -> Vec<LangValue> {
        self.input.clone()
    }

    /// A wrapper for [Self::input], to be called from wasm.
    #[cfg(feature = "wasm")]
    #[wasm_bindgen(getter, js_name = "output")]
    pub fn wasm_output(&self) -> Vec<LangValue> {
        self.output.clone()
    }

    /// A wrapper for [Self::registers], to be called from wasm. We can't send
    /// maps through wasm, so this returns a [JsValue] which is an object
    /// mapping register names (strings) to their values (`LangValue`).
    #[cfg(feature = "wasm")]
    #[wasm_bindgen(getter, js_name = "registers")]
    pub fn wasm_registers(&self) -> LangValueMap {
        // Convert the keys of the register map to strings
        let regs_by_name: HashMap<String, LangValue> = self
            .registers()
            .into_iter()
            .map(|(reg_ref, reg_value)| (reg_ref.to_string(), reg_value))
            .collect();
        // Convert the hashmap to a js object. Be careful here!
        JsValue::from_serde(&regs_by_name).unwrap().unchecked_into()
    }

    /// A wrapper for [Self::stacks], to be called from wasm. We can't send
    /// maps through wasm, so this returns a [JsValue] which is an object
    /// mapping stacks names (strings) to their values (`Vec<LangValue>`).
    #[cfg(feature = "wasm")]
    #[wasm_bindgen(getter, js_name = "stacks")]
    pub fn wasm_stacks(&self) -> LangValueArrayMap {
        // Convert the keys of the stacks map to strings
        let stacks_by_name: HashMap<String, &[LangValue]> = self
            .stacks()
            .into_iter()
            .map(|(stack_ref, stack_value)| {
                (stack_ref.to_string(), stack_value)
            })
            .collect();
        // Convert the hashmap to a js object. Be careful here!
        JsValue::from_serde(&stacks_by_name)
            .unwrap()
            .unchecked_into()
    }

    /// A wrapper for [Self::error], to be called from wasm. We can't send
    /// maps through wasm, so this returns a simplified error as a
    /// [SourceElement].
    #[cfg(feature = "wasm")]
    #[wasm_bindgen(getter, js_name = "error")]
    pub fn wasm_error(&self) -> Option<SourceElement> {
        self.error.as_ref().map(|wrapped_error| {
            // If an error is present, there should always be exactly one
            match wrapped_error.errors() {
                [error] => error.into(),
                errors => panic!(
                    "Expected exactly 1 runtime error, but got {:?}",
                    errors
                ),
            }
        })
    }

    /// A wrapper for [Self::execute_next], to be called from wasm. We throw
    /// away the error because it simplifies the logic on the TS side. That
    /// error is accessible via [Self::wasm_error] anyway.
    #[cfg(feature = "wasm")]
    #[doc(hidden)]
    #[wasm_bindgen(js_name = "executeNext")]
    pub fn wasm_execute_next(&mut self) -> bool {
        // If an error occurred, that means something executed, so return true
        self.execute_next().unwrap_or(true)
    }

    /// A wrapper for [Self::execute_all], to be called from wasm. We throw
    /// away the error because it simplifies the logic on the TS side. That
    /// error is accessible via [Self::wasm_error] anyway.
    #[cfg(feature = "wasm")]
    #[doc(hidden)]
    #[wasm_bindgen(js_name = "executeAll")]
    pub fn wasm_execute_all(&mut self) -> bool {
        // If an error occurred, that means something executed, so return true
        self.execute_all().unwrap_or(true)
    }
}
