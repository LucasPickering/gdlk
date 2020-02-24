#![deny(clippy::all, unused_must_use, unused_imports)]

use failure::Fallible;
use gdlk::{validator::Validate, Compiler, HardwareSpec, ProgramSpec, Valid};
use serde::de::DeserializeOwned;
use std::{fs, path::PathBuf, process};
use structopt::StructOpt;

/// The sub-command to execute.
#[derive(Debug, StructOpt)]
enum Command {
    /// Compile source code.
    #[structopt(name = "compile")]
    Compile {
        /// Path to the hardware spec file, in JSON format. If not provided, a
        /// default hardware spec will be used.
        #[structopt(parse(from_os_str), long = "hardware")]
        hardware_spec_path: Option<PathBuf>,
        /// Path to the source code file
        #[structopt(parse(from_os_str), long = "source", short = "s")]
        source_path: PathBuf,
    },

    /// Compile and execute source code.
    #[structopt(name = "run")]
    Run {
        /// Path to the hardware spec file, in JSON format. If not provided, a
        /// default hardware spec will be used.
        #[structopt(parse(from_os_str), long = "hardware")]
        hardware_spec_path: Option<PathBuf>,
        /// Path to the program spec file, in JSON format. If not provided, a
        /// default program spec will be used.
        #[structopt(parse(from_os_str), long = "program", short = "p")]
        program_spec_path: Option<PathBuf>,
        /// Path to the source code file
        #[structopt(parse(from_os_str), long = "source", short = "s")]
        source_path: PathBuf,
    },
}

/// GDLK executable, for compiling and executing GDLK programs
#[derive(Debug, StructOpt)]
#[structopt(name = "gdlk")]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

/// Loads a hardware or program spec from a file. If the path is None, returns
/// the default value instead.
fn load_spec<T: Default + DeserializeOwned + Validate>(
    path_opt: &Option<PathBuf>,
) -> Fallible<Valid<T>> {
    match path_opt {
        None => Ok(Valid::validate(T::default())?),
        Some(path) => {
            let spec_str = fs::read_to_string(path)?;
            Ok(Valid::validate(serde_json::from_str(&spec_str)?)?)
        }
    }
}

fn run(opt: Opt) -> Fallible<()> {
    match opt.cmd {
        // Compile and build the given program
        Command::Compile {
            hardware_spec_path,
            source_path,
        } => {
            let hw_spec: Valid<HardwareSpec> = load_spec(&hardware_spec_path)?;
            // Read the source code from the file
            let source = fs::read_to_string(source_path)?;
            // Compile
            Compiler::compile(source, hw_spec)?;
        }

        // Compile and build the given program
        Command::Run {
            hardware_spec_path,
            program_spec_path,
            source_path,
        } => {
            // Read and parse the hw spec and program spec from JSON files
            let hw_spec: Valid<HardwareSpec> = load_spec(&hardware_spec_path)?;
            let program_spec: Valid<ProgramSpec> =
                load_spec(&program_spec_path)?;

            // Read the source code from the file
            let source = fs::read_to_string(source_path)?;

            // Compile and execute
            let mut machine =
                Compiler::compile(source, hw_spec)?.allocate(&program_spec);
            let success = machine.execute_all()?;

            println!(
                "{}\nProgram completed with {}",
                serde_json::to_string_pretty(&machine)?,
                if success { "success" } else { "failure" },
            );
        }
    }
    Ok(())
}

fn main() {
    let exit_code = match run(Opt::from_args()) {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("{:#}", err);
            1
        }
    };
    process::exit(exit_code);
}
