use config::config_load;
use error::AppError;
use sytter::sytter_load;

mod config;
mod contrib;
mod error;
mod shell;
mod sytter;

fn main() -> Result<(), AppError> {
    let config = config_load()?;
    let sytter = sytter_load(&"somepath".to_string())?;
    sytter.watcher.watch_start(Box::new(move || {
        if sytter.condition.check_condition() {
            sytter.executor.execute()
                          .or_else(|| { sytter.failure.execute() })?;
        } else {
            sytter.failure.execute()?;
        }
    }))?;
    Ok(())
}
