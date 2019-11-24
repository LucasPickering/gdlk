use crate::{
    ast::{Environment, LangValue, MachineInstr, StackIdentifier},
    error::RuntimeError,
};
use failure::Fallible;
use std::iter;

/// Wrapper around a vec of stacks, to make it a bit easier to initialize/use.
///
/// All stack manipulation logic should be implemented here, to keep it
/// contained and scalable.
/// This is zero-cost at runtime, see [newtype pattern](https://doc.rust-lang.org/1.0.0/style/features/types/newtype.html).
#[derive(Debug, PartialEq)]
pub struct Stacks(Vec<Vec<LangValue>>);

impl Stacks {
    fn new(env: &Environment) -> Self {
        // Initialize `env.num_stacks` new stacks
        Self(iter::repeat_with(Vec::new).take(env.num_stacks).collect())
    }

    /// Gets a read-only view of the current set of stacks. Useful for
    /// visualizations and the like.
    pub fn get_all_stacks(&self) -> &[Vec<LangValue>] {
        &self.0
    }

    /// Gets the stack with the given ID. If the stack doesn't exist, returns an
    /// InvalidStackReference error.
    fn get_stack(
        &mut self,
        stack_id: StackIdentifier,
    ) -> Fallible<&mut Vec<LangValue>> {
        self.0
            .get_mut(stack_id)
            .ok_or_else(|| RuntimeError::InvalidStackReference(stack_id).into())
    }

    /// Pushes the given value onto the given stack. The environment is needed
    /// to compare the stack against a possible stack capacity rule. If the
    /// stack reference is invalid or the stack is at capacity, an error is
    /// returned.
    fn push(
        &mut self,
        env: &Environment,
        stack_id: StackIdentifier,
        value: LangValue,
    ) -> Fallible<()> {
        let stack = self.get_stack(stack_id)?;

        // If the stack is capacity, make sure we're not over it
        if let Some(max_stack_size) = env.max_stack_size {
            if stack.len() == max_stack_size {
                return Err(RuntimeError::StackOverflow(stack_id).into());
            }
        }

        stack.push(value);
        Ok(())
    }

    /// Pops an element off the given stack. If the pop is successful, the
    /// popped value is returned. If the stack doesn't exist or is empty, an
    /// error is returned.
    fn pop(&mut self, stack_id: StackIdentifier) -> Fallible<LangValue> {
        let stack = self.get_stack(stack_id)?;

        if let Some(val) = stack.pop() {
            Ok(val)
        } else {
            Err(RuntimeError::EmptyStack(stack_id).into())
        }
    }
}

/// The current state of a machine. This encompasses the entire state of a
/// program that is currently being executed.
///
/// The fields are public for read-only purposes. They should never be mutated.
/// Initialized from an [Environment](Environment), which controls the input
/// and stack parameters.
#[derive(Debug, PartialEq)]
pub struct MachineState {
    /// The current input buffer. This can be popped from as the program is
    /// executed. This will be initialized as the reverse of the input from the
    /// environment, so that elements at the beginning can be popped off first.
    /// Values can never be added to the input, only popped off.
    pub input: Vec<LangValue>,
    /// The current output buffer. This can be pushed into, but never popped
    /// out of.
    pub output: Vec<LangValue>,
    /// The mutable variable that the programmer has direct control over.
    pub workspace: LangValue,
    /// The series of stacks that act as the programs RAM. The number of stacks
    /// and their capacity is determined by the initialization environment.
    pub stacks: Stacks,
}

impl MachineState {
    fn new(env: &Environment) -> Self {
        let mut input = env.input.clone();
        input.reverse(); // Reverse the vec so we can pop off the values in order
        Self {
            stacks: Stacks::new(&env),
            input,
            output: Vec::new(),
            workspace: 0,
        }
    }
}

/// A steppable program executor. Maintains the current state of the program,
/// and execution can be progressed one instruction at a time.
///
/// Created from an [Environment](Environment) and a [Program](Program). The
/// current machine state can be obtained at any time, which allows for handy
/// visualizations of execution.
#[derive(Debug)]
pub struct Machine {
    // Static data
    env: Environment,
    program: Vec<MachineInstr>,
    // Runtime state
    state: MachineState,
    /// The index of the next instruction to be executed
    program_counter: usize,
}

impl Machine {
    /// Creates a new machine, ready to be executed.
    pub fn new(env: Environment, program: Vec<MachineInstr>) -> Self {
        let state = MachineState::new(&env);
        Self {
            env,
            program,
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
    pub fn execute_next(&mut self) -> Fallible<()> {
        let instr = self
            .program
            .get(self.program_counter)
            .expect("No instructions left to execute!");

        // Execute the instruction. For more instructions, the next pc is just
        // the next line, for those, return None and we'll populate the next
        // pc later. For jumps though, they need to specify a special next pc.
        let next_pc: Option<usize> = match instr {
            MachineInstr::Read => match self.state.input.pop() {
                Some(val) => {
                    self.state.workspace = val;
                    None
                }
                None => return Err(RuntimeError::EmptyInput.into()),
            },
            MachineInstr::Write => {
                self.state.output.push(self.state.workspace);
                None
            }
            MachineInstr::Set(val) => {
                self.state.workspace = *val;
                None
            }
            MachineInstr::Push(stack_id) => {
                self.state.stacks.push(
                    &self.env,
                    *stack_id,
                    self.state.workspace,
                )?;
                None
            }
            MachineInstr::Pop(stack_id) => {
                self.state.stacks.pop(*stack_id)?;
                None
            }
            MachineInstr::Jez(next_pc) => {
                if self.state.workspace == 0 {
                    Some(*next_pc)
                } else {
                    None
                }
            }
            MachineInstr::Jnz(next_pc) => {
                if self.state.workspace != 0 {
                    Some(*next_pc)
                } else {
                    None
                }
            }
        };

        // Advance the pc
        self.program_counter = next_pc.unwrap_or(self.program_counter + 1);
        Ok(())
    }

    /// Checks if this machine has finished executing.
    pub fn is_complete(&self) -> bool {
        self.program_counter >= self.program.len()
    }

    /// Checks if this machine has completed successfully. The criteria are:
    /// 1. Program is complete (all instructions have been executed)
    /// 2. Input buffer has been exhausted (all input has been consumed)
    /// 3. Output buffer matches the expected buffer, as defined by the
    /// [Environment](Environment)
    pub fn is_successful(&self) -> bool {
        self.is_complete()
            && self.state.input.is_empty()
            && self.state.output == self.env.expected_output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_program() {
        // Just a simple sanity check test
        let env = Environment {
            num_stacks: 0,
            max_stack_size: None,
            input: vec![1],
            expected_output: vec![1],
        };
        let program = vec![MachineInstr::Read, MachineInstr::Write];
        let mut machine = Machine::new(env, program);

        // Initial state
        assert_eq!(
            *machine.get_state(),
            MachineState {
                input: vec![1],
                output: vec![],
                workspace: 0,
                stacks: Stacks(vec![])
            }
        );
        assert!(!machine.is_successful());

        // Run one instruction
        machine.execute_next().unwrap();
        assert_eq!(
            *machine.get_state(),
            MachineState {
                input: vec![],
                output: vec![],
                workspace: 1,
                stacks: Stacks(vec![])
            }
        );
        assert!(!machine.is_successful());

        // Run the second instruction
        machine.execute_next().unwrap();
        assert_eq!(
            *machine.get_state(),
            MachineState {
                input: vec![],
                output: vec![1],
                workspace: 1,
                stacks: Stacks(vec![])
            }
        );
        assert!(machine.is_successful()); // Job's done
    }
}
