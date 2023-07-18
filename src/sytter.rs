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

pub struct Sytter {
    pub name: String,
    pub description: String,
    pub watcher: Box<dyn Watcher + Sync + Send>,
    pub condition: Box<dyn Condition + Sync + Send>,
    pub executor: Box<dyn Executor + Sync + Send>,
    pub failure: Box<dyn Failure + Sync + Send>,
}

pub fn sytter_load(_path: &String) -> Result<Sytter, AppError> {
    let sytter = Sytter {
        name: "Captive Portal Authentication".to_string(),
        description: "blah blah".to_string(),
        condition: Box::new(ShellCondition {
            shell: "".to_string(),
            expected_exit_codes: vec!(2),
        }),
        watcher: Box::new(CronWatch {
            cron: "5 * * * *".to_string(),
            shell: "".to_string(),
            watching: false,
        }),
        executor: Box::new(ShellExecutor {
            shell: "~/bin/captivate.sh".to_string(),
        }),
        failure: Box::new(ShellFailure {
            // repeat: false,
            shell: "printf '%s\n' 'Subject: Capitive Portal auth failed' \
                    '' \
                    'Captive Portal authentication is failing! See $LOG for \
                    details.' \
                    | sendmail $USER".to_string(),
        }),
    };
    Ok(sytter)
}

impl Sytter {
    pub fn start(mut self) -> Result<(), AppError> {
        let (join, sender) = self.watcher.watch_start(Box::new(move || {
            let res = if self.condition.check_condition() {
                self.executor
                    .execute()
                    .or_else(|_| { self.failure.execute() })
            } else {
                self.failure.execute()
            };
            match res {
                Ok(_) => (),
                Err(e) => println!("Error in watch loop: {:?}", e),
            }
            ()
        }))?;
        // TODO: Fix this.
        let _ = join.join();
        Ok(())
    }
}
