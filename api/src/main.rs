#![deny(clippy::all, unused_must_use, unused_imports)]
#![feature(const_fn, trait_alias, slice_patterns)]

// Diesel hasn't fully moved to Rust 2018 yet so we need this
#[macro_use]
extern crate diesel;

use actix_web::{middleware, App, HttpServer};
use diesel::{
    r2d2::{self, ConnectionManager},
    PgConnection,
};
use failure::Fallible;
use gdlk::{compile, HardwareSpec, ProgramSpec};
use std::{fs, path::PathBuf, process};
use structopt::StructOpt;

mod error;
mod models;
mod schema;
mod seed;
mod server;
mod util;
mod vfs;

#[derive(Debug, StructOpt)]
enum Command {
    /// Compile and execute source code. If execution terminates without error,
    /// returns 0 for a successful execution and 1 for unsuccessful.
    #[structopt(name = "execute")]
    Execute {
        /// Path to the hardware spec file, in JSON format
        #[structopt(parse(from_os_str), long = "hardware")]
        hardware_spec_path: PathBuf,
        /// Path to the program spec file, in JSON format
        #[structopt(parse(from_os_str), long = "program", short = "p")]
        program_spec_path: PathBuf,
        /// Path to the source code file
        #[structopt(parse(from_os_str), long = "source", short = "s")]
        source_path: PathBuf,
    },

    /// Start the HTTP server and listen for connections
    #[structopt(name = "server")]
    Server {
        /// The IP:port to bind to
        #[structopt(
            long,
            env = "SERVER_HOST",
            default_value = "localhost:8000"
        )]
        host: String,
    },

    /// Insert pre-defined seed data into the DB. Useful in development, not
    /// so much in release.
    #[structopt(name = "seed")]
    #[cfg(debug_assertions)]
    Seed,
}

/// GDLK executable, for executing GDLK programs or running the GDLK webserver
#[derive(Debug, StructOpt)]
#[structopt(name = "gdlk")]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
    /// Database URL
    #[structopt(long, env = "DATABASE_URL")]
    database_url: String,
}

fn run(opt: Opt) -> Fallible<()> {
    // Initialize the DB connection pool
    let manager = ConnectionManager::<PgConnection>::new(opt.database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    match opt.cmd {
        // Compile and build the given program
        Command::Execute {
            hardware_spec_path,
            program_spec_path,
            source_path,
        } => {
            // Read and parse the hw spec and program spec from JSON files
            let hardware_spec_str = fs::read_to_string(hardware_spec_path)?;
            let hardware_spec: HardwareSpec =
                serde_json::from_str(&hardware_spec_str)?;
            // Read and parse the hw spec from a JSON file
            let program_spec_str = fs::read_to_string(program_spec_path)?;
            let program_spec: ProgramSpec =
                serde_json::from_str(&program_spec_str)?;

            // Read the source code from the file
            let source = fs::read_to_string(source_path)?;

            // Compile and execute
            let mut machine = compile(&hardware_spec, &program_spec, source)?;
            let success = machine.execute_all()?;

            println!(
                "Program completed with {}",
                if success { "success" } else { "failure" }
            );
            process::exit(if success { 0 } else { 1 })
        }
        // Start the webserver
        Command::Server { host } => {
            // Set up logging
            std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
            env_logger::init();

            // Start the HTTP server
            HttpServer::new(move || {
                App::new()
                    // Need to clone because this init occurs once per thread
                    .data(pool.clone())
                    // enable logger
                    .wrap(middleware::Logger::default())
                    // routes
                    .service(server::file_system_get)
                    .service(server::ws_program_specs_by_id)
            })
            .bind(host)?
            .run()?;
        }
        Command::Seed => {
            let conn = pool.get().unwrap();
            seed::seed_db(&conn)?;
        }
    }
    Ok(())
}

fn main() {
    let result = run(Opt::from_args());
    if let Err(error) = result {
        eprintln!("Error!\n{:?}", error);
    }
}
