use async_trait::async_trait;
use crate::error::AppError;
use crate::trigger::Trigger;
use cron::Schedule;
use log::{debug, info};
use std::sync::mpsc::{Receiver, SyncSender};
use tokio_cron_scheduler::{Job, JobScheduler};

#[derive(Clone)]
pub struct CronWatch {
    pub cron: String,
}

#[async_trait]
impl Trigger for CronWatch {

    async fn trigger_await(
        &mut self,
        send_to_sytter: SyncSender<String>,
        _receive_from_sytter: Receiver<String>,
    ) -> Result<(), AppError> {
        let sched = JobScheduler::new()
            .await
            .map_err(|e| {
                AppError::TriggerInitializeError(
                    format!("JobScheduler could not start: {:?}", e),
                )
            })?;
        let send_to_sytter_threaded = send_to_sytter.clone();
        sched.add(
            Job::new(
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
                    info!("Cron trigger fired!");
                    match send_to_sytter_threaded.send("foo".to_string()) {
                        Ok(_) => debug!("Successfully sent message to Sytter."),
                        Err(e) => debug!("Error trigging to Sytter: {:?}", e),
                    };
                },
            )
            .map_err(|e| {
                AppError::TriggerInitializeError(
                    format!("Job could not be created: {:?}", e),
                )
            })?
        )
            .await
            .map_err(|e| {
                AppError::TriggerInitializeError(
                    format!("Job could not be added to scheduler: {:?}", e),
                )
            })
            ?;
        sched.start()
            .await
            .map_err(|e| {
                AppError::TriggerInitializeError(
                    format!("Could not start cron scheduler: {:?}", e),
                )
            })
            ?;
        Ok(())
    }

}
