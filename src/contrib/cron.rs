use async_trait::async_trait;
use cron::Schedule;
use tokio::runtime::Runtime;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use std::thread;
use std::sync::mpsc::{
    SyncSender,
    TryRecvError::{
        Empty,
        Disconnected,
    }
};

use tokio_cron_scheduler::{Job, JobScheduler};

use crate::error::AppError;
use crate::trigger::Trigger;

#[derive(Clone)]
pub struct CronWatch {
    pub cron: String,
}

#[async_trait]
impl Trigger for CronWatch {

    fn trigger_await(
        &mut self,
        send_to_sytter: SyncSender<String>,
        receive_from_sytter: Receiver<String>,
    ) -> Result<(), AppError> {
        let rt = Runtime::new()
            .map_err(|e| {
                AppError::TriggerInitializeError(
                    format!("Tokio Runtime could not start: {:?}", e),
                )
            })?;
        let sched = rt.block_on(JobScheduler::new())
            .map_err(|e| {
                AppError::TriggerInitializeError(
                    format!("JobScheduler could not start: {:?}", e),
                )
            })?;
        let send_to_sytter_threaded = send_to_sytter.clone();
        rt.block_on(sched.add(
            Job::new_cron_job(
                self.cron.parse::<Schedule>()
                    .map_err(|e| {
                        AppError::TriggerInitializeError(format!(
                            "Cron expression '{:?}' could not be parsed: {:?}",
                            self.cron,
                            e,
                        ))
                    })?
                    ,
                move |_uuid, _l| {
                    println!("Cron trigger fired!");
                    match send_to_sytter_threaded.send("foo".to_string()) {
                        Ok(_) => (),
                        Err(e) => println!("Error trigging to Sytter: {:?}", e),
                    };
                },
            )
            .map_err(|e| {
                AppError::TriggerInitializeError(
                    format!("Job could not be created: {:?}", e),
                )
            })?
        )
            )
            .map_err(|e| {
                AppError::TriggerInitializeError(
                    format!("Job could not be added to scheduler: {:?}", e),
                )
            })
            ?;
        // let mut ticks = 0;
        let looping = true;
        while looping {
            // Use a low sleep duration so we can shut down Sytter quickly.
            let duration = Duration::from_millis(1000);
            thread::sleep(duration);
            match receive_from_sytter.try_recv() {
                Ok(s) => println!("Received {:?} from sytter", s),
                Err(e) => match e {
                    Empty => (),
                    Disconnected => panic!(),
                },
            }
            println!("Loop done.");
        }
        Ok(())
    }

}
