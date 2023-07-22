use crate::{
    error::AppError,
    condition::Condition,
    contrib::{
        cron::CronWatch,
        shell::{ShellExecutor, ShellFailure, ShellCondition},
    },
    executor::Executor,
    failure::Failure,
    trigger::Trigger,
};
use std::{sync::mpsc::sync_channel, thread};

pub struct Sytter {
    pub name: String,
    pub description: String,
    pub watcher: Box<dyn Trigger + Sync + Send>,
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
            cron: "1/5 * * * * *".to_string(),
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
        let (send_to_trigger, receive_from_sytter) = sync_channel(0);
        let (send_to_sytter, receive_from_trigger) = sync_channel(0);
        let join = thread::spawn(move || {
            let res: Result<(), AppError> = self.watcher.trigger_await(
                send_to_sytter,
                receive_from_sytter,
            )
                .inspect_err(|e| {
                    println!("Error in watch loop: {:?}", e);
                })
                .and_then(|()| {
                    loop {
                        let trigger_message = receive_from_trigger.recv();
                        println!("Got trigger message: {:?}", trigger_message);
                        let res = self.condition
                            .check_condition()
                            .and_then(|cond| {
                                if cond {
                                    self.executor.execute()
                                } else {
                                    // Log that the condition fell through.
                                    Ok(())
                                }
                            })
                            .or_else(|e| {
                                self.failure.execute(e)
                            });
                        match res {
                            Ok(_) => (),
                            Err(e) => println!("It blowed up bad in condition! {:?}", e),
                        }
                    };
                });
            match res {
                Ok(_) => (),
                Err(e) => println!("It blowed up bad in trigger! {:?}", e),
            }
        });
        let _ = join.join();
        Ok(())
    }
}
