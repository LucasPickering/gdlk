#![deny(clippy::all)]

// Diesel hasn't fully moved to Rust 2018 yet so we need this
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate validator_derive;

pub mod config;
mod error;
pub mod models;
pub mod schema;
pub mod server;
pub mod util;
