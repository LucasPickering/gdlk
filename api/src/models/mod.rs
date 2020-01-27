//! All Diesel models live here.
//!
//! Some naming conventions for model structs:
//! - `Foo`: The "default" model, which contains all fields that in the DB
//!   table. This model should be the result of a `SELECT * ...`, and should
//!   never be constructed manually (outside of tests). Most query functions
//!   (e.g. filter helpers) should live on this type.
//! - `NewFoo`: A model that can be constructed in our Rust code, and inserted
//!   into the DB. It should generally define a `.insert()` helper.

mod hardware;
mod program;
mod user;

pub use hardware::*;
pub use program::*;
pub use user::*;
