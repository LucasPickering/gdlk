//! All Diesel models live here. One file per DB table.
//!
//! Some naming conventions for model structs:
//! - `Foo`: The "default" model, which contains all fields that in the DB
//!   table. This model should be the result of a `SELECT * ...`, and should
//!   never be constructed manually (outside of tests). Most query functions
//!   (e.g. filter helpers) should live on this type.
//! - `NewFoo`: A model that can be constructed in our Rust code, and inserted
//!   into the DB. It should generally define a `.insert()` helper, and
//!   implement the [Factory] trait.

mod hardware_spec;
mod permission;
mod program_spec;
mod role;
mod role_permission;
mod user;
mod user_program;
mod user_program_record;
mod user_provider;
mod user_role;

pub use hardware_spec::*;
pub use permission::*;
pub use program_spec::*;
pub use role::*;
pub use role_permission::*;
pub use user::*;
pub use user_program::*;
pub use user_program_record::*;
pub use user_provider::*;
pub use user_role::*;
