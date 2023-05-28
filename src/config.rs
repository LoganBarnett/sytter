use crate::contrib::shell::{ShellExecutor, ShellFailure};
use crate::executor::Executor;
use crate::failure::Failure;
use crate::{error, contrib::shell::ShellCondition};
use crate::watcher::Watcher;
use crate::condition::Condition;
use error::AppError;
use crate::contrib::cron::CronWatch;

pub struct Config<'a> {
    pub sytters_path: &String,
}

pub fn config_load<'a>() -> Result<Config<'a>, AppError> {
}
