use config::config_load;
use error::AppError;
use sytter::sytter_load;

mod condition;
mod config;
mod contrib;
mod error;
mod failure;
mod executor;
mod shell;
mod sytter;
mod watcher;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let _config = config_load()?;
    let sytter = sytter_load(&"somepath".to_string())?;
    sytter.start()
}
