use failure::Fallible;
use gdlk_api::{config::GdlkConfig, util};
use log::{debug, info};

fn run() -> Fallible<()> {
    env_logger::init();

    let config = GdlkConfig::load()?;
    info!("Loaded config");
    debug!("{:#?}", &config);

    let pool = util::init_db_conn_pool()?;
    Ok(gdlk_api::server::run_server(config, pool)?)
}

fn main() {
    let result = run();
    if let Err(error) = result {
        eprintln!("Error!\n{:?}", error);
    }
}
