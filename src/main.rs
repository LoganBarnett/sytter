// Enables use of Result::inspect_err, a method which allows one to perform an
// effect on a Result (this is the Err case) without changing anything about the
// Result.
#![feature(result_option_inspect)]
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
mod trigger;


#[tokio::main]
async fn main() -> Result<(), AppError> {
    let _config = config_load()?;
    let sytter = sytter_load(&"somepath".to_string())?;
    sytter.start().await
}
