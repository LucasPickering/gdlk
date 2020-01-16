//! All code related to the webserver. Basically anything that calls Actix
//! lives here.

mod fs;
mod websocket;

use actix_web::{middleware, App, HttpServer};
use diesel::{r2d2::ConnectionManager, PgConnection};
use std::io;

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[actix_rt::main]
pub async fn run_server(pool: Pool, host: String) -> io::Result<()> {
    // Set up logging
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    // Start the HTTP server
    HttpServer::new(move || {
        App::new()
            // Need to clone because this init occurs once per thread
            .data(pool.clone())
            // enable logger
            .wrap(middleware::Logger::default())
            // routes
            .service(fs::file_system_get)
            .service(websocket::ws_program_specs_by_slugs)
    })
    .bind(host)?
    .run()
    .await
}
