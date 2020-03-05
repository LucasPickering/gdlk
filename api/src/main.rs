#![deny(clippy::all, unused_must_use, unused_imports)]
// Need to allow this because Diesel's macros violate it
#![allow(clippy::single_component_path_imports)]
#![feature(const_fn)]

// Diesel hasn't fully moved to Rust 2018 yet so we need this
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate validator_derive;

use diesel::{
    r2d2::{self, ConnectionManager},
    PgConnection,
};
use failure::Fallible;
use structopt::StructOpt;

mod error;
mod models;
mod schema;
#[cfg(debug_assertions)]
mod seed;
mod server;
mod util;
// Commenting this out so we don't have to maintain it - we may want to come
// back to it at some point
// mod vfs;

/// The sub-command to execute.
#[derive(Debug, StructOpt)]
enum Command {
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
        // Start the webserver
        Command::Server { host } => {
            server::run_server(pool, host)?;
        }
        #[cfg(debug_assertions)]
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
