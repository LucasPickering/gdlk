//! All Diesel models live here. One file per DB table.
//!
//! Some naming conventions for model structs:
//! - `Foo`: The "default" model, which contains all fields that in the DB
//!   table. This model should be the result of a `SELECT * ...`, and should
//!   never be constructed manually (outside of tests). Most query functions
//!   (e.g. filter helpers) should live on this type.
//! - `NewFoo`: A model that can be constructed in our Rust code, and inserted
//!   into the DB. It should generally define a `.insert()` helper.

mod hardware_spec;
mod program_spec;
mod user;
mod user_program;

pub use hardware_spec::*;
pub use program_spec::*;
pub use user::*;
pub use user_program::*;
