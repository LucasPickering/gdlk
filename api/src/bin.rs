use failure::Fallible;
use gdlk_api::util;
use std::env;

// Commenting this out so we don't have to maintain it - we may want to come
// back to it at some point
// mod vfs;

fn run() -> Fallible<()> {
    env_logger::init();
    let server_host =
        env::var("SERVER_HOST").unwrap_or_else(|_| "localhost:8000".into());
    let pool = util::init_db_conn_pool()?;
    Ok(gdlk_api::server::run_server(pool, server_host)?)
}

fn main() {
    let result = run();
    if let Err(error) = result {
        eprintln!("Error!\n{:?}", error);
    }
}
