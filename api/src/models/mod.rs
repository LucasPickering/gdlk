//! All Diesel models live here. Some naming conventions for model structs:
//! - `Full*`: A model that contains all fields that in the DB table. This model
//!   should be the result of a `SELECT * ...`, and should never be constructed
//!   manually (outside of tests)
//! - `New*`: A model that can be constructed in our Rust code, and inserted
//!   into the DB.

mod hardware;
mod program;

pub use hardware::*;
pub use program::*;
