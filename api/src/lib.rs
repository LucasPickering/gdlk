#![deny(clippy::all, unused_must_use, unused_imports)]
// Need to allow this because Diesel's macros violate it
#![allow(clippy::single_component_path_imports)]
#![feature(const_fn)]

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
