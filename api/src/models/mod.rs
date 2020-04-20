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
mod program_spec;
mod user;
mod user_program;

use diesel::PgConnection;
pub use hardware_spec::*;
pub use program_spec::*;
pub use user::*;
pub use user_program::*;

/// A trait that makes it easy to generate a row for a particular type. This
/// should only be used for tests.
pub trait Factory {
    /// The type returned from an insert of this type.
    type ReturnType;

    /// Insert this object into the DB and return the full DB row.
    fn create(self, conn: &PgConnection) -> Self::ReturnType;
}
