use crate::{error::AppError, trigger::Trigger};
use cron::Schedule;
use log::{debug, info, trace};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, SyncSender};
use job_scheduler_ng::{Job, JobScheduler};
use toml::Table;
use tap::Tap;

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[typetag::serde]
impl Trigger for CronTrigger {
  fn trigger_await(
    &mut self,
    send_to_sytter: SyncSender<String>,
    _receive_from_sytter: Receiver<String>,
  ) -> Result<(), AppError> {
    let mut sched = JobScheduler::new();
    // let send_to_sytter_threaded = send_to_sytter.clone();
    sched
      .add(
        Job::new(
          self.cron.parse::<Schedule>().map_err(|e| {
            AppError::TriggerInitializeError(format!(
              "Cron expression '{:?}' could not be parsed: {:?}",
              self.cron, e,
            ))
          })?,
          move || {
            info!("Cron trigger fired!");
            // We don't really have a meaningful message to send, I
            // think. Not yet. For now we just need to send _something_.
            match send_to_sytter.send("foo".to_string()) {
              Ok(_) => {
                debug!("Successfully sent message to Sytter.")
              }
              Err(e) => {
                debug!("Error triggering to Sytter: {:#}", e)
              }
            };
          },
        )
          // .map_err(|e| {
          //   AppError::TriggerInitializeError(format!(
          //     "Job could not be created: {:?}",
          //     e
          //   ))
          // })?,
      )
      ;
    // .map_err(|e| {
    //     AppError::TriggerInitializeError(format!(
    //         "Job could not be added to scheduler: {:?}",
    //         e
    //     ))
    // })?;
    // sched.start().map_err(|e| {
    //     AppError::TriggerInitializeError(format!(
    //         "Could not start cron scheduler: {:?}",
    //         e
    //     ))
    // })?;
    loop {
      sched.tick();
      std::thread::sleep(
        sched
          .time_till_next_job()
          .tap(|t| trace!("Time until next cron tick: {:?}ms", t))
      );
    }
    // Ok(())
  }
}
