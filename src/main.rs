// Enables use of Result::inspect_err, a method which allows one to perform an
// effect on a Result (this is the Err case) without changing anything about the
// Result.
#![feature(result_option_inspect)]
use config::{env_config_load, config_cli_merge, cli_parse};
use error::AppError;
use log::debug;
use logging::logger_init;
use sytter::sytter_load;

mod condition;
mod config;
mod contrib;
mod error;
mod failure;
mod executor;
mod logging;
mod shell;
mod sytter;
mod trigger;


#[tokio::main]
async fn main() -> Result<(), AppError> {
    let env_config = env_config_load()?;
    let cli_config = cli_parse()?;
    let config = config_cli_merge(env_config, cli_config);
    logger_init(config.verbosity.log_level())?;
    debug!("Using config: {:?}", config);
    let sytter = sytter_load(&config.sytters_path)?;
    sytter.start().await
}
