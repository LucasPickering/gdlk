#![deny(clippy::all, unused_must_use)]

use failure::Fallible;
use std::io::Read;

mod ast;
mod error;
mod machine;
mod parse;
mod util;

pub use crate::{
    ast::{Environment, LangValue, Program},
    machine::{Machine, MachineState, Stacks},
};

/// Reads source code from the given [`Read`](Read), validates it, and executes
/// it. Returns `true` if the program was successful, `false` if not.
pub fn run_program(env: Environment, source: &mut impl Read) -> Fallible<bool> {
    let program = Program::parse(source)?;
    let mut machine = Machine::new(&env, &program);

    while let Some(instr) = &mut machine.execute_next()? {
        // this hullabaloo is necessary because of lifetimes
        let instr_str = format!("{:?}", instr);
        println!("Executed {}; State: {:?}", instr_str, machine.get_state());
    }
    println!(
        "Program terminated with {}",
        if machine.is_successful() {
            "success"
        } else {
            "failure"
        }
    );

    Ok(machine.is_successful())
}
