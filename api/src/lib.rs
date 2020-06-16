#![deny(clippy::all, unused_must_use, unused_imports)]

// Diesel hasn't fully moved to Rust 2018 yet so we need this
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate validator_derive;

mod error;
pub mod models;
pub mod schema;
pub mod server;
pub mod util;
