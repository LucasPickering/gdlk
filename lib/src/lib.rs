#![deny(clippy::all, unused_must_use)]

use failure::Fallible;
use std::io::Read;

mod ast;
mod compiler;
mod error;
mod machine;
mod util;

use crate::compiler::Compiler;

pub use crate::{
    ast::{Environment, LangValue, Program},
    machine::{Machine, MachineState, Stacks},
};

/// Compiles the given source program, with the given environment, into a
/// [Machine](Machine). The returned machine can then be executed.
pub fn compile(env: Environment, source: &mut impl Read) -> Fallible<Machine> {
    Ok(Compiler::new()
        .parse(source)?
        .desugar()
        .delabel()
        .compile(env))
}

/// Reads source code from the given [`Read`](Read), validates it, and executes
/// it. Returns `true` if the program was successful, `false` if not.
pub fn run_program(env: Environment, source: &mut impl Read) -> Fallible<bool> {
    let mut machine = compile(env, source)?;

    while !machine.is_complete() {
        machine.execute_next()?;
        println!("Executed instruction. State: {:?}", machine.get_state());
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
