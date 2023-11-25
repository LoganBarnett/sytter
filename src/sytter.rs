use crate::{
    condition::Condition,
    contrib::{
        cron::cron_trigger_toml_deserialize,
        power::power_trigger_toml_deserialize,
        shell::{
            shell_condition_toml_deserialize, shell_executor_toml_deserialize,
            shell_failure_toml_deserialize,
        },
    },
    error::AppError,
    executor::Executor,
    failure::Failure,
    trigger::Trigger,
};
use log::*;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::read_to_string,
    path::Path,
    sync::mpsc::{sync_channel, Receiver},
};
use toml::Table;

#[derive(Debug)]
pub struct Sytter {
    pub name: String,
    pub description: String,
    pub triggers: Vec<Box<dyn Trigger>>,
    pub conditions: Vec<Box<dyn Condition>>,
    pub executors: Vec<Box<dyn Executor>>,
    pub failures: Vec<Box<dyn Failure>>,
}

#[derive(Debug, Deserialize)]
pub struct SytterSection {
    pub kind: String,
    pub settings: HashMap<String, String>,
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
) -> Result<Box<dyn Trigger>, AppError> {
    let kind = section_data.get("kind").and_then(|x| x.as_str()).ok_or(
        AppError::SytterDeserializeRawError(
            "Field 'kind' missing from Trigger.".to_string(),
        ),
    )?;
    match kind {
        "cron" => cron_trigger_toml_deserialize(section_data),
        "power" => power_trigger_toml_deserialize(section_data),
        _ => Err(AppError::SytterDeserializeRawError(format!(
            "Kind '{}' not supported",
            kind,
        ))),
    }
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
    let triggers: Vec<Box<dyn Trigger>> = sd
        .triggers
        .iter()
        .map(sytter_trigger_table_deserialize)
        .collect::<Result<Vec<Box<dyn Trigger>>, AppError>>()?;
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
    fn trigger_execute_on_message(
        &mut self,
        // _send_to_sytter: SyncSender<String>,
        receive_from_trigger: &Receiver<String>,
    ) -> Result<(), AppError> {
        info!("Waiting for message from trigger...");
        let trigger_message = receive_from_trigger.recv();
        debug!("Got trigger message: {:?}", trigger_message);
        self.conditions
            .get(0)
            .ok_or(AppError::SytterMissingComponentError(
                "No conditions!".to_string(),
            ))?
            .check_condition()
            .and_then(|cond| {
                if cond {
                    debug!("Conditional is true, executing...");
                    self.executors
                        .get(0)
                        .ok_or(AppError::SytterMissingComponentError(
                            "No executors!".to_string(),
                        ))?
                        .execute()
                        .inspect(|_| info!("Execution successful."))
                } else {
                    debug!("Conditional is false.");
                    // Log that the condition fell through.
                    Ok(())
                }
            })
            .or_else(|e| {
                error!("Executor failed: {:?}", e);
                self.failures
                    .get(0)
                    .ok_or(AppError::SytterMissingComponentError(
                        "No failures!".to_string(),
                    ))?
                    .execute(e)
                    .inspect(|_| {
                        info!("Failure handler successful!");
                    })
            })
            .map(|_| ())
            .inspect_err(|e| error!("It blowed up bad in condition! {:?}", e))
    }

    pub async fn start(mut self) -> Result<(), AppError> {
        // Eventually we want to signal the trigger that it should close any
        // resources it is using because the Sytter is being unloaded.
        let (_send_to_trigger, receive_from_sytter) = sync_channel::<String>(0);
        let (send_to_sytter, receive_from_trigger) = sync_channel(0);
        let trigger: &mut Box<dyn Trigger> = self.triggers.get_mut(0).ok_or(
            AppError::SytterMissingComponentError("No triggers!".to_string()),
        )?;

        let res: Result<(), AppError> = trigger
            .trigger_await(send_to_sytter, receive_from_sytter)
            .await
            .inspect_err(|e| {
                error!("Error in watch loop: {:?}", e);
            })
            .and_then(|()| loop {
                let _ = self.trigger_execute_on_message(&receive_from_trigger);
            });
        match res {
            Ok(_) => (),
            Err(e) => error!("It blowed up bad in trigger! {:?}", e),
        }
        Ok(())
    }
}
