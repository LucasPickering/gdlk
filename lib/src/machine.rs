use crate::{
    ast::{Environment, Instruction, LangValue, Program, StackIdentifier},
    error::RuntimeError,
};
use failure::Fallible;

/// Wrapper around a vec of stacks, to make it a bit easier to initialize/use.
///
/// All stack manipulation logic should be implemented here, to keep it
/// contained and scalable.
/// This is zero-cost at runtime, see [newtype pattern](https://doc.rust-lang.org/1.0.0/style/features/types/newtype.html).
#[derive(Debug, PartialEq)]
pub struct Stacks(Vec<Vec<LangValue>>);

impl Stacks {
    fn new(env: &Environment) -> Self {
        // Initialize x new stacks, where x is env.num_stacks
        Self((0..env.num_stacks).map(|_| Vec::new()).collect())
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

/// A recursive structure to track the progression of a program. A program
/// counter consist of a flat list of instructions that need to be run, plus
/// an optional child `ProgramCounter`. If the child is present, the child's
/// instructions will always be executed before any of the parent's. Once all
/// of a program counter's instructions have been executed, it's considered
/// exhausted.
#[derive(Debug)]
struct ProgramCounter<'a> {
    child: Option<Box<Self>>,
    instructions: &'a [Instruction],
}

impl<'a> ProgramCounter<'a> {
    /// Creates a new `ProgramCounter` with the given instructions and no child.
    fn new(body: &'a [Instruction]) -> Self {
        Self {
            child: None,
            instructions: body,
        }
    }

    /// Adds a child to this program counter. If this PC already has a child,
    /// then the new body will be passed down the family tree until a childless
    /// PC is found, at which point it will be stored.
    fn add_child(&mut self, body: &'a [Instruction]) {
        if let Some(child) = &mut self.child {
            // If we already have a child, pass the new one down the chain
            child.add_child(body);
        } else {
            // We don't have a child yet, store this one
            self.child = Some(Box::new(Self::new(body)));
        }
    }

    /// Gets the next executable instruction from this PC. If a child is
    /// present, calls down to it first, then falls back on our own list of
    /// instructions. This does _not_ modify the PC, so subsequent calls to
    /// `get_next()` will return the same instruction. If no more instructions
    /// are available, that means this PC is exhausted, so return `None`.
    fn get_next(&self) -> Option<&'a Instruction> {
        if let Some(child) = &self.child {
            // We have a child, call down to it
            let child_opt = child.get_next();

            // A child should never be exhausted (enforced in advance()). Do a
            // safety check here just to be safe though.
            if child_opt.is_none() {
                panic!("child.get_next() returned None");
            }

            child_opt
        } else {
            // We are the innermost expression, go to the next instruction
            self.instructions.get(0)
        }
    }

    /// Advance the PC by one instruction. This removes the foremost instruction
    /// from the PC, so that it is no longer executable. If this PC is
    /// exhausted, then this will panic!. Don't do that!
    fn advance(&mut self) {
        if let Some(child) = &mut self.child {
            child.advance();

            // If the child is exhausted, we can throw it away
            if child.is_exhausted() {
                self.child = None
            }
        }

        if !self.instructions.is_empty() {
            self.instructions = &self.instructions[1..];
        } else {
            // This case indicates a bug in the calling code, so we want a panic
            panic!("Cannot advance an exhausted program counter");
        }
    }

    /// Checks if this PC has been exhausted, meaning that it has no more
    /// executable instructions.
    fn is_exhausted(&self) -> bool {
        self.get_next().is_none()
    }
}

/// A steppable program executor. Maintains the current state of the program,
/// and execution can be progressed one instruction at a time.
///
/// Created from an [Environment](Environment) and a [Program](Program). The
/// current machine state can be obtained at any time, which allows for handy
/// visualizations of execution.
#[derive(Debug)]
pub struct Machine<'a> {
    // Static data
    env: &'a Environment,
    // It'd be nice to be able to store the Program here too, but that creates
    // lifetime headaches with ProgramCounter. Punting for now, will figure
    // this out later if we need it

    // Runtime state
    state: MachineState,
    program_counter: ProgramCounter<'a>,
}

impl<'a> Machine<'a> {
    /// Creates a new machine, ready to be executed.
    pub fn new(env: &'a Environment, program: &'a Program) -> Self {
        let state = MachineState::new(&env);
        Self {
            env,
            state,
            program_counter: ProgramCounter::new(&program.body),
        }
    }

    /// Gets an immutable reference to the current machine state.
    pub fn get_state(&self) -> &MachineState {
        &self.state
    }

    /// Executes the given instruction, modifying internal state as needed. This
    /// will advance the program counter after execution, if necessary. In some
    /// cases, advancing the PC is not necessarily, which is when an instruction
    /// should be executed again. This occurs with loops.
    fn execute_instruction(&mut self, instr: &'a Instruction) -> Fallible<()> {
        // Run the instruction, for most instructions, execution exhausts the
        // instruction, so exhausted=true. But for WHILE, we may need to repeat
        // the instruction, so it might not be exhausted.
        let exhausted = match instr {
            Instruction::Read => match self.state.input.pop() {
                Some(val) => {
                    self.state.workspace = val;
                    true
                }
                None => return Err(RuntimeError::EmptyInput.into()),
            },
            Instruction::Write => {
                self.state.output.push(self.state.workspace);
                true
            }
            Instruction::Set(val) => {
                self.state.workspace = *val;
                true
            }
            Instruction::Push(stack_id) => {
                self.state.stacks.push(
                    &self.env,
                    *stack_id,
                    self.state.workspace,
                )?;
                true
            }
            Instruction::Pop(stack_id) => {
                self.state.stacks.pop(*stack_id)?;
                true
            }
            Instruction::If(body) => {
                if self.state.workspace != 0 {
                    self.program_counter.add_child(body)
                }
                true
            }
            Instruction::While(body) => {
                if self.state.workspace != 0 {
                    self.program_counter.add_child(body);
                    // We'll have to check the condition again after the child
                    // is exhausted, so don't exhaust this WHILE instruction yet
                    false
                } else {
                    true
                }
            }
        };

        // If we're done with this instruction, advance to the next one.
        if exhausted {
            self.program_counter.advance();
        }
        Ok(())
    }

    /// Executes the next instruction in the program, i.e. "take one step".
    /// Returns the executed instruction. If there are no more instructions to
    /// execute, returns `Ok(None)`. If an error occurs, returns `Err`.
    pub fn execute_next(&mut self) -> Fallible<Option<&Instruction>> {
        if let Some(instr) = self.program_counter.get_next() {
            // Run the instruction, and check if it emitted a new body
            self.execute_instruction(instr)?;
            Ok(Some(instr))
        } else {
            // No more instructions left
            Ok(None)
        }
    }

    /// Checks if this machine has successfully completed. The criteria are:
    /// 1. Program has been exhausted (all instructions have been executed)
    /// 2. Input buffer has been exhausted (all input has been consumed)
    /// 3. Output buffer matches the expected buffer, as defined by the
    /// [Environment](Environment)
    pub fn is_successful(&self) -> bool {
        self.program_counter.is_exhausted()
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
        let program = Program {
            body: vec![Instruction::Read, Instruction::Write],
        };
        let mut machine = Machine::new(&env, &program);

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
        assert_eq!(machine.execute_next().unwrap(), Some(&Instruction::Read));
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
        assert_eq!(machine.execute_next().unwrap(), Some(&Instruction::Write));
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
