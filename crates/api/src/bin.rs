use gdlk_api::config::GdlkConfig;
use log::{debug, info};

fn main() {
    env_logger::init();

    let config = GdlkConfig::load().unwrap();
    info!("Loaded config");
    debug!("{:#?}", &config);

    gdlk_api::server::run_server(config).unwrap();
}
