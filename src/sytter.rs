use crate::{
    error::AppError,
    condition::Condition,
    contrib::{
        cron::CronWatch,
        shell::{ShellExecutor, ShellFailure, ShellCondition},
    },
    executor::Executor,
    failure::Failure,
    watcher::Watcher,
};
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;

pub struct Sytter<'a> {
    pub name: String,
    pub description: String,
    pub watcher: Arc<Mutex<&'a mut (dyn Watcher + Sync + Send)>>,
    pub condition: RefCell<&'a (dyn Condition + Sync)>,
    pub executor: RefCell<&'a (dyn Executor + Sync)>,
    pub failure: RefCell<&'a (dyn Failure + Sync)>,
}

// Somehow we can't return this.
// I need a Rust cookbook or something, because stuff like this comes up all the
// time and answers aren't clear.
pub fn sytter_load<'a>(_path: &String) -> Result<Sytter<'a>, AppError> {
    let mut watcher = Arc::new(Mutex::new(&mut CronWatch {
        cron: "5 * * * *".to_string(),
        shell: "".to_string(),
        watching: false,
    }));
    let executor = RefCell::new(&ShellExecutor {
        shell: "~/bin/captivate.sh".to_string(),
    });
    let failure = RefCell::new(&ShellFailure {
        // repeat: false,
        shell: "printf '%s\n' 'Subject: Capitive Portal auth failed' \
                 '' \
                 'Captive Portal authentication is failing! See $LOG for details.' \
                 | sendmail $USER".to_string(),
    });
    let condition = RefCell::new(&ShellCondition {
        shell: "".to_string(),
        expected_exit_codes: vec!(2),
    });
    let sytter = Sytter {
        name: "Captive Portal Authentication".to_string(),
        description: "blah blah".to_string(),
        condition: condition,
        watcher: watcher,
        executor: executor,
        failure: failure,
    };
    Ok(sytter)

}
