use crate::{error::AppError, trigger::Trigger};
use async_trait::async_trait;
use cron::Schedule;
use log::{debug, info};
use serde::Deserialize;
use std::sync::mpsc::{Receiver, SyncSender};
use tokio_cron_scheduler::{Job, JobScheduler};
use toml::Table;

#[derive(Clone, Debug, Deserialize)]
pub struct CronTrigger {
    pub cron: String,
}

pub fn cron_trigger_toml_deserialize(
    section_data: &Table,
) -> Result<Box<dyn Trigger>, AppError> {
    Ok(Box::new(CronTrigger {
        cron: section_data
            .get("cron")
            .and_then(|x| x.as_str())
            .ok_or(AppError::SytterDeserializeRawError(
                "Field 'cron' missing from Trigger.".to_string(),
            ))?
            .to_string(),
    }))
}

#[async_trait]
impl Trigger for CronTrigger {
    async fn trigger_await(
        &mut self,
        send_to_sytter: SyncSender<String>,
        _receive_from_sytter: Receiver<String>,
    ) -> Result<(), AppError> {
        let sched = JobScheduler::new().await.map_err(|e| {
            AppError::TriggerInitializeError(format!(
                "JobScheduler could not start: {:?}",
                e,
            ))
        })?;
        let send_to_sytter_threaded = send_to_sytter.clone();
        sched
            .add(
                Job::new(
                    self.cron.parse::<Schedule>().map_err(|e| {
                        AppError::TriggerInitializeError(format!(
                            "Cron expression '{:?}' could not be parsed: {:?}",
                            self.cron, e,
                        ))
                    })?,
                    move |_uuid, _l| {
                        info!("Cron trigger fired!");
                        // We don't really have a meaningful message to send, I
                        // think. Not yet. For now we just need to send _something_.
                        match send_to_sytter_threaded.send("foo".to_string()) {
                            Ok(_) => {
                                debug!("Successfully sent message to Sytter.")
                            }
                            Err(e) => {
                                debug!("Error trigging to Sytter: {:?}", e)
                            }
                        };
                    },
                )
                .map_err(|e| {
                    AppError::TriggerInitializeError(format!(
                        "Job could not be created: {:?}",
                        e
                    ))
                })?,
            )
            .await
            .map_err(|e| {
                AppError::TriggerInitializeError(format!(
                    "Job could not be added to scheduler: {:?}",
                    e
                ))
            })?;
        sched.start().await.map_err(|e| {
            AppError::TriggerInitializeError(format!(
                "Could not start cron scheduler: {:?}",
                e
            ))
        })?;
        Ok(())
    }
}
