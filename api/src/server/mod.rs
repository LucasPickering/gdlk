mod fs;
mod websocket;

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

// Everything exported from here should be a route handler
use diesel::{r2d2::ConnectionManager, PgConnection};
pub use fs::*;
pub use websocket::*;
