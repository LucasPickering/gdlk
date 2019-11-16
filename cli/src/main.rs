#![feature(box_syntax)]

use failure::Error;
use gdlk::{self, Environment};
use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};
use structopt::StructOpt;

/// GDLK command-line runner.
#[derive(StructOpt, Debug)]
#[structopt(name = "gdlkc")]
struct Opt {
    /// Input file, to read source code from. If not specified, will read
    /// from stdin.
    #[structopt(parse(from_os_str), long = "input", short = "i")]
    input: Option<PathBuf>,
}

fn run(opt: Opt) -> Result<(), Error> {
    // Get the source to read from, either a file or stdin
    let mut input_source: Box<dyn Read> = if let Some(input_path) = opt.input {
        box File::open(&input_path)?
    } else {
        box io::stdin()
    };

    gdlk::run_program(
        Environment {
            num_stacks: 0,
            max_stack_size: None,
            input: vec![1, 2, 3],
            expected_output: vec![2, 3, 4],
        },
        &mut input_source,
    )?;
    Ok(())
}

fn main() {
    let result = run(Opt::from_args());
    if let Err(error) = result {
        eprintln!("Error!\n{:?}", error);
    }
}
