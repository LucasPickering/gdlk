#![deny(clippy::all, unused_must_use)]

// Diesel hasn't fully moved to Rust 2018 yet so we need this
#[macro_use]
extern crate diesel;

use actix_web::{middleware, web, App, HttpServer};
use diesel::{
    r2d2::{self, ConnectionManager},
    PgConnection,
};

mod error;
mod lang;
mod models;
mod schema;
mod server;
mod util;

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    let manager = ConnectionManager::<PgConnection>::new(
        std::env::var("DATABASE_URL").unwrap(),
    );
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            // Need to clone because this init occurs once per thread
            .data(pool.clone())
            // enable logger
            .wrap(middleware::Logger::default())
            // websocket route
            .service(
                web::resource("/ws/environments/{env_id}/")
                    .route(web::get().to(server::ws_environments_by_id)),
            )
    })
    // start http server
    .bind(std::env::var("SERVER_HOST").unwrap())?
    .run()
}
