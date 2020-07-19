use failure::Fallible;
use gdlk_api::config::GdlkConfig;
use log::{debug, info};

fn run() -> Fallible<()> {
    env_logger::init();

    let config = GdlkConfig::load()?;
    info!("Loaded config");
    debug!("{:#?}", &config);

    Ok(gdlk_api::server::run_server(config)?)
}

fn main() {
    let result = run();
    if let Err(error) = result {
        eprintln!("Error!\n{:?}", error);
    }
}
