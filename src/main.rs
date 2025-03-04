use std::{path::{Path, PathBuf}, sync::{Arc, Mutex}};

use config::{cli_parse, config_cli_merge, env_config_load};
use error::AppError;
use log::*;
use logging::logger_init;
use sytter::sytter_load;
use http_server::http_server;

use crate::state::State;

mod condition;
mod config;
mod contrib;
mod deserialize;
mod error;
mod executor;
mod failure;
mod http_server;
mod logging;
#[cfg(target_os = "macos")]
mod macos;
// #[cfg(target_os = "macos")]
// mod macos_bindings;
mod shell;
mod sytter;
mod trigger;
mod state;

fn sytter_paths(base_path: &String) -> Result<Vec<PathBuf>, AppError> {
    let path = Path::new(base_path);
    Ok(if path.is_dir() {
        From::from(
            path.read_dir()
                .and_then(|dir| {
                    dir.map(|res| res.map(|entry| path.join(entry.path())))
                        .collect::<Result<Vec<PathBuf>, _>>()
                })
                .map_err(AppError::SyttersDirInvalidError)?,
        )
    } else {
        vec![path.to_path_buf()]
    })
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
  let env_config = env_config_load()?;
  let cli_config = cli_parse()?;
  let config = config_cli_merge(env_config, cli_config);
  logger_init(config.verbosity.log_level())?;
  debug!("Using config: {:?}", config);
  for file in sytter_paths(&config.sytters_path)? {
    info!("Starting sytter '{}'...", file.display());
    let config_copy = config.clone();
    std::thread::spawn(move || {
      // TODO: Handle errors from results (await and load).
      let sytter = sytter_load(file.as_path())
        .inspect(|s| debug!("Loaded Sytter: {:?}", s))
        .expect(&format!("Failed to load sytter '{}'.", file.display()));
      sytter.start(&config_copy);
    });
  }
  http_server().await?;
    Ok(())
}
