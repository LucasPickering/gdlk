#![deny(clippy::all, unused_must_use, unused_imports)]
#![feature(try_trait)]

// Diesel hasn't fully moved to Rust 2018 yet so we need this
#[macro_use]
extern crate diesel;

use crate::{lang::compile, models::Environment};
use actix_web::{middleware, web, App, HttpServer};
use diesel::{
    r2d2::{self, ConnectionManager},
    PgConnection,
};
use failure::Fallible;
use std::{fs, path::PathBuf, process};
use structopt::StructOpt;

mod error;
mod lang;
mod models;
mod schema;
mod server;
mod util;

#[derive(Debug, StructOpt)]
enum Command {
    /// Compile and execute source code. If execution terminates without error,
    /// returns 0 for a successful execution and 1 for unsuccessful.
    #[structopt(name = "execute")]
    Execute {
        /// Path to the environment file, in JSON format
        #[structopt(parse(from_os_str), long = "env", short = "e")]
        env_path: PathBuf,
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
        /// Database URL
        #[structopt(long, env = "DATABASE_URL")]
        database_url: String,
    },
}

/// GDLK executable, for executing GDLK programs or running the GDLK webserver
#[derive(Debug, StructOpt)]
#[structopt(name = "gdlk")]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

fn run(opt: Opt) -> Fallible<()> {
    match opt.cmd {
        // Compile and build the given program
        Command::Execute {
            env_path,
            source_path,
        } => {
            // Read and parse the environment from a JSON file
            let env_str = fs::read_to_string(env_path)?;
            let env: Environment = serde_json::from_str(&env_str)?;

            // Read the source code from the file
            let source = fs::read_to_string(source_path)?;

            // Compile and execute
            let mut machine = compile(&env, source)?;
            let success = machine.execute_all()?;

            println!(
                "Program completed with {}",
                if success { "success" } else { "failure" }
            );
            process::exit(if success { 0 } else { 1 })
        }
        // Start the webserver
        Command::Server { host, database_url } => {
            // Set up logging
            std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
            env_logger::init();

            // Initialize the DB connection pool
            let manager = ConnectionManager::<PgConnection>::new(database_url);
            let pool = r2d2::Pool::builder()
                .build(manager)
                .expect("Failed to create pool.");

            // Start the HTTP server
            HttpServer::new(move || {
                App::new()
                    // Need to clone because this init occurs once per thread
                    .data(pool.clone())
                    // enable logger
                    .wrap(middleware::Logger::default())
                    // websocket route
                    .service(
                        web::resource("/ws/environments/{env_id}/").route(
                            web::get().to(server::ws_environments_by_id),
                        ),
                    )
            })
            .bind(host)?
            .run()?;
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
