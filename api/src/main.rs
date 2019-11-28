#![deny(clippy::all, unused_must_use)]

use actix_web::{middleware, web, App, HttpServer};

mod error;
mod lang;
mod models;
mod server;
mod util;

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            // websocket route
            .service(
                web::resource("/ws/").route(web::get().to(server::ws_index)),
            )
    })
    // start http server
    .bind("127.0.0.1:8080")?
    .run()
}
