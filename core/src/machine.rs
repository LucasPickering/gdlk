use crate::{
    ast::{
        LangValue, MachineInstr, Operator, RegisterRef, StackIdentifier,
        ValueSource,
    },
    debug,
    error::RuntimeError,
    models::{HardwareSpec, ProgramSpec},
};
use serde::Serialize;
use std::{cmp::Ordering, convert::TryFrom, iter, num::Wrapping};

/// The current state of a machine. This encompasses the entire state of a
/// program that is currently being executed.
///
/// The fields are public for read-only purposes. They should never be mutated.
/// Initialized from an [HardwareSpec](HardwareSpec) and
/// [ProgramSpec](ProgramSpec), which control the input and stack parameters.
#[derive(Debug, PartialEq, Serialize)]
pub struct MachineState {
    /// The maximum number of elements allowed in a stack. This is copied from
    /// the hardware spec so we don't have to maintain a reference to the hw.
    #[serde(skip)] // We don't want to include this field in serialization
    max_stack_length: usize,

    /// The current input buffer. This can be popped from as the program is
    /// executed. This will be initialized as the reverse of the input from the
    /// program spec, so that elements at the beginning can be popped off
    /// first. Values can never be added to the input, only popped off.
    pub input: Vec<LangValue>,
    /// The current output buffer. This can be pushed into, but never popped
    /// out of.
    pub output: Vec<LangValue>,
    /// The registers that the user can read and write. Indexed by Register ID.
    pub registers: Vec<LangValue>,
    /// The series of stacks that act as the programs RAM. The number of stacks
    /// and their capacity is determined by the initializating hardware spec.
    pub stacks: Vec<Vec<LangValue>>,
}

impl MachineState {
    fn new(hardware_spec: &HardwareSpec, program_spec: &ProgramSpec) -> Self {
        let mut input = program_spec.input.clone();
        input.reverse(); // Reverse the vec so we can pop off the values in order
        Self {
            max_stack_length: hardware_spec.max_stack_length,
            input,
            output: Vec::new(),
            registers: iter::repeat(0)
                .take(usize::try_from(hardware_spec.num_registers).unwrap())
                .collect(),
            stacks: iter::repeat_with(Vec::new)
                .take(usize::try_from(hardware_spec.num_stacks).unwrap())
                .collect(),
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
            // TODO make this conversion safe
            RegisterRef::InputLength => self.input.len() as LangValue,
            RegisterRef::StackLength(stack_id) => {
                // TODO safe num conversion here
                self.stacks[*stack_id].len() as LangValue
            }
            RegisterRef::User(reg_id) => *self.registers.get(*reg_id).unwrap(),
        }
    }

    /// Sets the register to the given value. The register reference is
    /// assumed to be valid (should be validated at build time). Will panic if
    /// it isn't valid.
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
}

/// A steppable program executor. Maintains the current state of the program,
/// and execution can be progressed one instruction at a time.
///
/// Created from a [HardwareSpec](HardwareSpec), [ProgramSpec](ProgramSpec), and
/// a program. The current machine state can be obtained at any time, which
/// allows for handy visualizations of execution.
#[derive(Debug)]
pub struct Machine {
    // Static data
    program: Vec<MachineInstr>,
    expected_output: Vec<LangValue>,
    // Runtime state
    state: MachineState,
    /// The index of the next instruction to be executed
    program_counter: usize,
}

impl Machine {
    /// Creates a new machine, ready to be executed.
    pub fn new(
        hardware_spec: &HardwareSpec,
        program_spec: &ProgramSpec,
        program: Vec<MachineInstr>,
    ) -> Self {
        let state = MachineState::new(hardware_spec, program_spec);
        Self {
            program,
            // This is a punt but w/e
            expected_output: program_spec.expected_output.clone(),
            state,
            program_counter: 0,
        }
    }

    /// Gets an immutable reference to the current machine state.
    pub fn get_state(&self) -> &MachineState {
        &self.state
    }

    /// Executes the next instruction in the program. If there are no
    /// instructions left to execute, this panics.
    pub fn execute_next(&mut self) -> Result<(), RuntimeError> {
        let instr = self
            .program
            .get(self.program_counter)
            .ok_or(RuntimeError::ProgramTerminated)?;

        // Execute the instruction. For most instructions, the number of
        // instructions to consume is just 1, so for those return None. For
        // jumps though, it can vary so they'll return Some(n).
        let instrs_to_consume: Option<i32> = match instr {
            MachineInstr::Operator(op) => {
                match op {
                    Operator::Read(reg) => match self.state.input.pop() {
                        Some(val) => {
                            self.state.set_reg(reg, val);
                        }
                        None => return Err(RuntimeError::EmptyInput),
                    },
                    Operator::Write(reg) => {
                        self.state.output.push(self.state.get_reg(reg));
                    }
                    Operator::Set(dst, src) => {
                        self.state.set_reg(
                            dst,
                            self.state.get_value_from_source(src),
                        );
                    }
                    Operator::Add(dst, src) => {
                        self.state.set_reg(
                            dst,
                            (Wrapping(self.state.get_reg(dst))
                                + Wrapping(
                                    self.state.get_value_from_source(src),
                                ))
                            .0,
                        );
                    }
                    Operator::Sub(dst, src) => {
                        self.state.set_reg(
                            dst,
                            (Wrapping(self.state.get_reg(dst))
                                - Wrapping(
                                    self.state.get_value_from_source(src),
                                ))
                            .0,
                        );
                    }
                    Operator::Mul(dst, src) => {
                        self.state.set_reg(
                            dst,
                            (Wrapping(self.state.get_reg(dst))
                                * Wrapping(
                                    self.state.get_value_from_source(src),
                                ))
                            .0,
                        );
                    }
                    Operator::Cmp(dst, src_1, src_2) => {
                        let val_1 = self.state.get_value_from_source(src_1);
                        let val_2 = self.state.get_value_from_source(src_2);
                        let cmp = match val_1.cmp(&val_2) {
                            Ordering::Less => -1,
                            Ordering::Equal => 0,
                            Ordering::Greater => 1,
                        };
                        self.state.set_reg(dst, cmp);
                    }
                    Operator::Push(src, stack_id) => {
                        self.state.push_stack(
                            *stack_id,
                            self.state.get_value_from_source(src),
                        )?;
                    }
                    Operator::Pop(stack_id, dst) => {
                        let popped = self.state.pop_stack(*stack_id)?;
                        self.state.set_reg(dst, popped);
                    }
                }
                None
            }

            MachineInstr::Jez(num_instrs, reg) => {
                if self.state.get_reg(reg) == 0 {
                    Some(*num_instrs)
                } else {
                    None
                }
            }
            MachineInstr::Jnz(num_instrs, reg) => {
                if self.state.get_reg(reg) != 0 {
                    Some(*num_instrs)
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
        debug!(println!("Executed {:?}; State: {:?}", instr, self.state));
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
    /// 3. Output buffer matches the expected buffer, as defined by the
    /// [ProgramSpec](ProgramSpec)
    pub fn is_successful(&self) -> bool {
        self.is_complete()
            && self.state.input.is_empty()
            && self.state.output == self.expected_output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_program() {
        // Just a simple sanity check test
        let hardware_spec = HardwareSpec::default();
        let program_spec = ProgramSpec {
            input: vec![1],
            expected_output: vec![1],
        };
        let program = vec![
            MachineInstr::Operator(Operator::Read(RegisterRef::User(0))),
            MachineInstr::Operator(Operator::Write(RegisterRef::User(0))),
        ];
        let mut machine = Machine::new(&hardware_spec, &program_spec, program);

        // Initial state
        assert_eq!(
            *machine.get_state(),
            MachineState {
                max_stack_length: 0,
                input: vec![1],
                output: vec![],
                registers: vec![0],
                stacks: vec![],
            }
        );
        assert!(!machine.is_successful());

        // Run one instruction
        machine.execute_next().unwrap();
        assert_eq!(
            *machine.get_state(),
            MachineState {
                max_stack_length: 0,
                input: vec![],
                output: vec![],
                registers: vec![1],
                stacks: vec![],
            }
        );
        assert!(!machine.is_successful());

        // Run the second instruction
        machine.execute_next().unwrap();
        assert_eq!(
            *machine.get_state(),
            MachineState {
                max_stack_length: 0,
                input: vec![],
                output: vec![1],
                registers: vec![1],
                stacks: vec![],
            }
        );
        assert!(machine.is_successful()); // Job's done
    }
}
