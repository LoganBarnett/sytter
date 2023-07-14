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

fn main() -> Result<(), AppError> {
    let _config = config_load()?;
    let mut sytter = &mut sytter_load(&"somepath".to_string())?;
    let executor = &sytter.executor;
    let condition = &sytter.condition;
    let failure = &sytter.failure;
    let mut watcher_mutex = &mut sytter.watcher;
    if let Ok(watcher) = watcher_mutex.lock() {
    watcher.watch_start(Box::new(move || {
        if condition.borrow().check_condition() {
            executor
                  .borrow()
                  .execute()
                  .or_else(|_| { failure.borrow().execute() });
        } else {
            failure.borrow().execute();
        }
    }))?;
    }
    Ok(())
}
