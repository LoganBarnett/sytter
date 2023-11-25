// Enables use of Result::inspect_err, a method which allows one to perform an
// effect on a Result (this is the Err case) without changing anything about the
// Result.
#![feature(result_option_inspect)]
use std::path::{Path, PathBuf};

use config::{cli_parse, config_cli_merge, env_config_load};
use error::AppError;
use log::debug;
use logging::logger_init;
use sytter::sytter_load;

extern crate num;
#[macro_use]
extern crate num_derive;

#[cfg(target_os = "macos")]
mod macos_bindings;
mod condition;
mod config;
mod contrib;
mod deserialize;
mod error;
mod executor;
mod failure;
mod logging;
#[cfg(target_os = "macos")]
mod power_macos;
mod shell;
mod sytter;
mod trigger;

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
        let sytter = sytter_load(file.as_path())?;
        sytter.start().await?
    }
    Ok(())
}
