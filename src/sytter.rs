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
            expected_exit_codes: vec!(0),
            script: "ls".to_string(),
            shell: "bash".to_string(),
        }),
        watcher: Box::new(CronWatch {
            cron: "1/5 * * * * *".to_string(),
        }),
        executor: Box::new(ShellExecutor {
            shell: "bash".to_string(),
            script: "~/bin/captivate.sh".to_string(),
        }),
        failure: Box::new(ShellFailure {
            script: "printf '%s\n' 'Subject: Capitive Portal auth failed' \
                    '' \
                    'Captive Portal authentication is failing! See $LOG for \
                    details.' \
                    | sendmail $USER".to_string(),
            shell: "bash".to_string(),
        }),
    };
    Ok(sytter)
}

impl Sytter {

    pub async fn start(mut self) -> Result<(), AppError> {
        let (send_to_trigger, receive_from_sytter) = sync_channel(0);
        let (send_to_sytter, receive_from_trigger) = sync_channel(0);
        let res: Result<(), AppError> = self.watcher.trigger_await(
            send_to_sytter,
            receive_from_sytter,
        )
            .await
            .inspect_err(|e| {
                println!("Error in watch loop: {:?}", e);
            })
            .and_then(|()| {
                loop {
                    println!("Waiting for message from trigger...");
                    let trigger_message = receive_from_trigger.recv();
                    println!("Got trigger message: {:?}", trigger_message);
                    let _ = self
                        .condition
                        .check_condition()
                        .and_then(|cond| {
                            if cond {
                                println!("Conditional is true, executing...");
                                self.executor
                                    .execute()
                                    .inspect(|_| { println!("Execution successful.") })
                            } else {
                                println!("Conditional is false.");
                                // Log that the condition fell through.
                                Ok(())
                            }
                        })
                        .or_else(|e| {
                            println!("Executor failed: {:?}", e);
                            self.failure.execute(e)
                                        .inspect(|_| {
                                            println!("Failure handler successful!");
                                        })
                        })
                        .map(|_| { () } )
                        .inspect_err(|e| {
                            println!("It blowed up bad in condition! {:?}", e)
                        });
                };
            });
        match res {
            Ok(_) => (),
            Err(e) => println!("It blowed up bad in trigger! {:?}", e),
        }
        Ok(())
    }
}
