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
use std::env;

mod error;
mod models;
mod schema;
mod server;
mod util;
// Commenting this out so we don't have to maintain it - we may want to come
// back to it at some point
// mod vfs;

fn run() -> Fallible<()> {
    let database_url = env::var("DATABASE_URL")?;
    let server_host =
        env::var("SERVER_HOST").unwrap_or_else(|_| "localhost:8000".into());

    // Initialize the DB connection pool
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    Ok(server::run_server(pool, server_host)?)
}

fn main() {
    let result = run();
    if let Err(error) = result {
        eprintln!("Error!\n{:?}", error);
    }
}
