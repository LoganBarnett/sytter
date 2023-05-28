use crate::{error::AppError, contrib::{cron::CronWatch, shell::{ShellExecutor, ShellFailure, ShellCondition}}};

pub trait Condition {
   fn check_condition(&self) -> bool;
}

pub trait Failure {
    fn execute(&self) -> Result<(), AppError>;
}

pub trait Executor {
    fn execute(&self) -> Result<(), AppError>;
}

pub trait Watcher {
    fn watch_start(
        &mut self,
        watch_trigger: Box<dyn Fn() + Send + 'static>,
    ) -> Result<(), AppError>;
    fn watch_stop(&mut self) -> Result<(), AppError>;
}

pub struct Sytter<'a> {
    pub name: String,
    pub description: String,
    pub watcher: &'a dyn Watcher,
    pub condition: &'a dyn Condition,
    pub executor: &'a dyn Executor,
    pub failure: &'a dyn Failure,
}

pub fn sytter_load<'a>(path: &String) -> Result<Sytter<'a>, AppError> {
    let watcher = CronWatch {
        cron: "5 * * * *".to_string(),
        shell: "".to_string(),
        watching: false,
    };
    let executor = ShellExecutor {
        shell: "~/bin/captivate.sh".to_string(),
    };
    let failure = ShellFailure {
        // repeat: false,
        shell: "printf '%s\n' 'Subject: Capitive Portal auth failed' \
                 '' \
                 'Captive Portal authentication is failing! See $LOG for details.' \
                 | sendmail $USER".to_string(),
    };
    let condition = ShellCondition {
        shell: "".to_string(),
        expected_exit_codes: vec!(2),
    };
    let sytter = Sytter {
        name: "Captive Portal Authentication".to_string(),
        description: "blah blah".to_string(),
        condition: &condition,
        watcher: &watcher,
        executor: &executor,
        failure: &failure,
    };
    Ok(sytter)

}
