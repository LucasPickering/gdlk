#![feature(box_syntax)]

use failure::Error;
use gdlk;
use std::{
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
};
use structopt::StructOpt;

/// GDLK command-line compiler.
#[derive(StructOpt, Debug)]
#[structopt(name = "gdlkc")]
enum Opt {
    /// Compiles source code
    #[structopt(name = "compile")]
    Compile {
        /// Input file, to read source code from. If not specified, will read
        /// from stdin.
        #[structopt(parse(from_os_str), long = "input", short = "i")]
        input: Option<PathBuf>,

        /// Output file, to write assembly code to. If not specified, will
        /// write to stdout.
        #[structopt(parse(from_os_str), long = "output", short = "o")]
        output: Option<PathBuf>,
    },
}

fn run(opt: Opt) -> Result<(), Error> {
    match opt {
        Opt::Compile { input, output } => {
            // Get the source to read from, either a file or stdin
            let mut input_source: Box<dyn Read> =
                if let Some(input_path) = input {
                    box File::open(&input_path)?
                } else {
                    box io::stdin()
                };

            // Get the destination to write to, either a file or stdout
            // TODO: currently this opens the file immediately, so it will be
            // open during the entire compilation
            let mut output_dest: Box<dyn Write> =
                if let Some(output_path) = output {
                    box File::open(&output_path)?
                } else {
                    box io::stdout()
                };
            gdlk::compile(&mut input_source, &mut output_dest)?;
            Ok(())
        }
    }
}

fn main() {
    let result = run(Opt::from_args());
    if let Err(error) = result {
        eprintln!("Error!\n{:?}", error);
    }
}
