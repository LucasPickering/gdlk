//! Views are a layer on top of models that hold as much logic as possible
//! for fetching, mutations, etc. They're an abstraction layer between the
//! models and the GQL responders.

mod hardware_spec;
mod program_spec;
mod user;
mod user_program;

use crate::error::ResponseResult;
pub use hardware_spec::*;
pub use program_spec::*;
pub use user::*;
pub use user_program::*;

/// A `View` is a type that can perform some action which is called from the
/// API. It could be a read or a write operation. The struct should hold
/// whatever context is necessary to perform the operation, and it can be
/// executed using [Self::execute].
pub trait View {
    type Output;

    fn execute(&self) -> ResponseResult<Self::Output>;
}
