use crate::{
    condition::Condition, config::Config, contrib::{
        cron::cron_trigger_toml_deserialize,
        device::device_connection_toml_deserialize,
        power::power_trigger_toml_deserialize,
        shell::{
            shell_condition_toml_deserialize, shell_executor_toml_deserialize,
            shell_failure_toml_deserialize,
        },
    }, error::AppError, executor::Executor, failure::Failure, trigger::Trigger
};
use log::*;
use serde::Deserialize;
use std::{
    fs::read_to_string,
    path::Path,
    sync::{mpsc::{sync_channel, Receiver, SyncSender}, Arc, Mutex},
};
use toml::Table;

#[derive(Debug)]
pub struct Sytter {
  #[allow(unused)]
    pub name: String,
  #[allow(unused)]
    pub description: String,
    pub triggers: Vec<Arc<Mutex<Box<dyn Trigger>>>>,
    pub conditions: Vec<Box<dyn Condition>>,
    pub executors: Vec<Box<dyn Executor>>,
    pub failures: Vec<Box<dyn Failure>>,
}

#[derive(Debug, Deserialize)]
pub struct SytterDeserializedRaw {
    pub name: String,
    pub description: String,
    #[serde(rename = "trigger")]
    pub triggers: Vec<Table>,
    #[serde(rename = "condition")]
    pub conditions: Vec<Table>,
    #[serde(rename = "execute")]
    pub executors: Vec<Table>,
    #[serde(rename = "failure")]
    pub failures: Vec<Table>,
}

pub fn sytter_trigger_table_deserialize(
    section_data: &Table,
) -> Result<Arc<Mutex<Box<dyn Trigger>>>, AppError> {
    let kind = section_data.get("kind").and_then(|x| x.as_str()).ok_or(
        AppError::SytterDeserializeRawError(
            "Field 'kind' missing from Trigger.".to_string(),
        ),
    )?;
  Ok(Arc::new(Mutex::new(
    match kind {
      "cron" => cron_trigger_toml_deserialize(section_data),
      "device-connection" => device_connection_toml_deserialize(section_data),
      "power" => power_trigger_toml_deserialize(section_data),
      _ => Err(AppError::SytterDeserializeRawError(format!(
        "Kind '{}' not supported",
        kind,
      ))),
    }?
  )))
}

pub fn sytter_condition_table_deserialize(
    section_data: &Table,
) -> Result<Box<dyn Condition>, AppError> {
    let kind = section_data.get("kind").and_then(|x| x.as_str()).ok_or(
        AppError::SytterDeserializeRawError(
            "Field 'kind' missing from Condition.".to_string(),
        ),
    )?;
    if kind == "shell" {
        shell_condition_toml_deserialize(section_data)
    } else {
        Err(AppError::SytterDeserializeRawError(format!(
            "Kind '{}' not supported",
            kind,
        )))
    }
}

pub fn sytter_executor_table_deserialize(
    section_data: &Table,
) -> Result<Box<dyn Executor>, AppError> {
    let kind = section_data.get("kind").and_then(|x| x.as_str()).ok_or(
        AppError::SytterDeserializeRawError(
            "Field 'kind' missing from Executor.".to_string(),
        ),
    )?;
    if kind == "shell" {
        shell_executor_toml_deserialize(section_data)
    } else {
        Err(AppError::SytterDeserializeRawError(format!(
            "Kind '{}' not supported",
            kind,
        )))
    }
}

pub fn sytter_failure_table_deserialize(
    section_data: &Table,
) -> Result<Box<dyn Failure>, AppError> {
    let kind = section_data.get("kind").and_then(|x| x.as_str()).ok_or(
        AppError::SytterDeserializeRawError(
            "Field 'kind' missing from Failure.".to_string(),
        ),
    )?;
    if kind == "shell" {
        shell_failure_toml_deserialize(section_data)
    } else {
        Err(AppError::SytterDeserializeRawError(format!(
            "Kind '{}' not supported",
            kind,
        )))
    }
}

// How to dynamically load "plugins" for this is not well understood, and how
// traits get deserialized is also not well supported or documented. So we just
// create a full Sytter here.
pub fn sytter_deserialize(
    sd: SytterDeserializedRaw,
) -> Result<Sytter, AppError> {
    let triggers: Vec<Arc<Mutex<Box<dyn Trigger>>>> = sd
        .triggers
        .iter()
        .map(sytter_trigger_table_deserialize)
        .collect::<Result<Vec<Arc<Mutex<Box<dyn Trigger>>>>, AppError>>()?;
    let conditions: Vec<Box<dyn Condition>> = sd
        .conditions
        .iter()
        .map(sytter_condition_table_deserialize)
        .collect::<Result<Vec<Box<dyn Condition>>, AppError>>()?;
    let executors: Vec<Box<dyn Executor>> = sd
        .executors
        .iter()
        .map(sytter_executor_table_deserialize)
        .collect::<Result<Vec<Box<dyn Executor>>, AppError>>()?;
    let failures: Vec<Box<dyn Failure>> = sd
        .failures
        .iter()
        .map(sytter_failure_table_deserialize)
        .collect::<Result<Vec<Box<dyn Failure>>, AppError>>()?;
    Ok(Sytter {
        conditions,
        description: sd.description,
        executors,
        failures,
        name: sd.name,
        triggers,
    })
}

pub fn sytter_load(path: &Path) -> Result<Sytter, AppError> {
    let sd = toml::from_str::<SytterDeserializedRaw>(
        &read_to_string(path).map_err(AppError::SytterReadError)?,
    )
    .map_err(AppError::SytterDeserializeError)?;
    sytter_deserialize(sd).inspect(|s| debug!("Deserialized Sytter: {:?}", s))
}

impl Sytter {

  pub fn start(self, config: &Config) -> Result<(), AppError> {
    if self.triggers.len() == 0 {
      error!("{}: No triggers found.  Nothing to do here.", self.name);
      Err(AppError::TriggersMissing(self.name.clone()))
    } else {
      let name = self.name.clone();
      // TODO: Wire up error handling properly.
      let triggers_copy = self.triggers.clone();
      let _channels = triggers_copy.clone().into_iter().map(move |trigger| {
        let config_copy = config.clone();
        let name_copy = name.clone();
        let threaded_sytter = ThreadedSytter {
          name: name_copy.clone(),
          description: self.description.clone(),
          conditions: self.conditions.clone(),
          executors: self.executors.clone(),
          failures: self.failures.clone(),
        };
        std::thread::spawn(move || {
          let (send_to_trigger, receive_from_sytter) = sync_channel::<String>(0);
          let (send_to_sytter, receive_from_trigger) = sync_channel(0);
          let _join_handle = std::thread::spawn(move || {
            trace!("Awaiting trigger for {}", name_copy);
            let binding_trigger = trigger.clone();
            binding_trigger
              .lock()
              .unwrap()
              .trigger_await(send_to_sytter, receive_from_sytter)
              .unwrap()
              ;
          });
          loop {
            let _ = threaded_sytter.trigger_execute_on_message(
              &config_copy,
              &send_to_trigger,
              &receive_from_trigger,
            );
          }
        });
        Ok(())
      })
        .collect::<Result<Vec<()>, AppError>>()?;
      Ok(())
    }
  }

}

// It's kind of hard to pass self around into a thread.  I can't figure it out.
// So let's just create a new type that doesn't have triggers (because we don't
// need them), and we'll copy everything there and use that.
#[derive(Debug)]
pub struct ThreadedSytter {
  pub name: String,
  pub description: String,
  pub conditions: Vec<Box<dyn Condition>>,
  pub executors: Vec<Box<dyn Executor>>,
  pub failures: Vec<Box<dyn Failure>>,
}

impl ThreadedSytter {

  fn trigger_execute_on_message(
    &self,
    config: &Config,
    _send_to_trigger: &SyncSender<String>,
    receive_from_trigger: &Receiver<String>,
  ) -> Result<(), AppError> {
    info!("{}: Waiting for message from trigger...", self.name);
    let trigger_message = receive_from_trigger.recv();
    debug!("{}: Got trigger message: {:?}", self.name, trigger_message);
    self
      .conditions
      .get(0)
      .ok_or(AppError::SytterMissingComponentError(
        "No conditions!".into(),
      ))?
      .check_condition(&config)
      .and_then(|cond| {
        if cond {
          debug!("{}: Conditional is true, executing...", self.name);
          self
            .executors
            .get(0)
            .ok_or(AppError::SytterMissingComponentError(
              format!("No executors in {}!", self.name).into(),
            ))?
            .execute(config)
            .inspect(|_| debug!("{}: Execution successful.", self.name))
        } else {
          debug!("{}: Conditional is false.", self.name);
          // Log that the condition fell through.
          Ok(())
        }
      })
      .or_else(|e| {
        error!("{}: Executor failed: {:?}", self.name, e);
        self.failures
          .get(0)
          .ok_or(AppError::SytterMissingComponentError(
            "No failures!".to_string(),
          ))?
          .execute(&config, e)
          .inspect(|_| {
            info!("{}: Failure handler successful!", self.name);
          })
      })
      .map(|_| ())
      .inspect_err(|e| error!("It blowed up bad in condition! {:?}", e))
  }

}
