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
use log::{
    debug,
    error,
    info,
};
use serde::{Deserialize};
use toml::{Table, Value};
use std::{
    sync::mpsc::sync_channel,
    path::Path,
    fs::read_to_string,
    collections::HashMap,
    vec,
};

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
    let kind = section_data
        .get("kind")
        .and_then(|x| x.as_str())
        .ok_or(AppError::SytterDeserializeRawError(
            "Field 'kind' missing from Trigger.".to_string(),
        ))?
        ;
    if kind == "cron" {
        Ok(Box::new(CronWatch {
            cron: section_data
                .get("cron")
                .and_then(|x| x.as_str())
                .ok_or(AppError::SytterDeserializeRawError(
                    "Field 'cron' missing from Trigger.".to_string(),
                ))?
                .to_string(),
        }))
    } else {
        Err(AppError::SytterDeserializeRawError(format!(
            "Kind '{}' not supported",
            kind,
        )))
    }
}

pub fn i32_des(x: &Value) -> Option<i32> {
    x.as_integer().map(|xx| -> i32 {
        xx as i32
    })
}

pub fn vec_i32_des(o: Option<&Value>) -> Vec<i32> {
    o
        .and_then(|x| x.as_array())
        .and_then(|ys| -> Option<Vec<i32>> {
            ys
                .iter()
                .map(i32_des)
                .collect::<Option<Vec<i32>>>()
        })
        .unwrap_or(vec!(0_i32))
}

pub fn sytter_condition_table_deserialize(
    section_data: &Table,
) -> Result<Box<dyn Condition>, AppError> {
    let kind = section_data
        .get("kind")
        .and_then(|x| x.as_str())
        .ok_or(AppError::SytterDeserializeRawError(
            "Field 'kind' missing from Condition.".to_string(),
        ))?
        ;
    if kind == "shell" {
        Ok(Box::new(ShellCondition {
            script: section_data
                .get("script")
                .and_then(|x| x.as_str())
                .ok_or(AppError::SytterDeserializeRawError(
                    "Field 'script' missing from Condition.".to_string(),
                ))?
                .to_string(),
            shell: section_data
                .get("shell")
                .and_then(|x| x.as_str())
                .map(|x| x.to_string())
                .unwrap_or("/bin/bash".to_string())
                ,
            expected_exit_codes: vec_i32_des(
                section_data.get("expected_exit_codes"),
            ),
        }))
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
    let kind = section_data
        .get("kind")
        .and_then(|x| x.as_str())
        .ok_or(AppError::SytterDeserializeRawError(
            "Field 'kind' missing from Executor.".to_string(),
        ))?
        ;
    if kind == "shell" {
        Ok(Box::new(ShellExecutor {
            expected_exit_codes: vec_i32_des(
                section_data.get("expected_exit_codes"),
            ),
            script: section_data
                .get("script")
                .and_then(|x| x.as_str())
                .ok_or(AppError::SytterDeserializeRawError(
                    "Field 'script' missing from Executor.".to_string(),
                ))?
                .to_string(),
            shell: section_data
                .get("shell")
                .and_then(|x| x.as_str())
                .map(|x| x.to_string())
                .unwrap_or("/bin/bash".to_string())
                ,
        }))
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
    let kind = section_data
        .get("kind")
        .and_then(|x| x.as_str())
        .ok_or(AppError::SytterDeserializeRawError(
            "Field 'kind' missing from Failure.".to_string(),
        ))?
        ;
    if kind == "shell" {
        Ok(Box::new(ShellFailure {
            expected_exit_codes: vec_i32_des(
                section_data.get("expected_exit_codes"),
            ),
            script: section_data
                .get("script")
                .and_then(|x| x.as_str())
                .ok_or(AppError::SytterDeserializeRawError(
                    "Field 'script' missing from Failure.".to_string(),
                ))?
                .to_string(),
            shell: section_data
                .get("shell")
                .and_then(|x| x.as_str())
                .map(|x| x.to_string())
                .unwrap_or("/bin/bash".to_string())
                ,
        }))
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
    let triggers: Vec<Box<dyn Trigger>> = sd.triggers
        .iter()
        .map(sytter_trigger_table_deserialize)
        .collect::<Result<Vec<Box<dyn Trigger>>, AppError>>()
        ?;
    let conditions: Vec<Box<dyn Condition>> = sd.conditions
        .iter()
        .map(sytter_condition_table_deserialize)
        .collect::<Result<Vec<Box<dyn Condition>>, AppError>>()
        ?;
    let executors: Vec<Box<dyn Executor>> = sd.executors
        .iter()
        .map(sytter_executor_table_deserialize)
        .collect::<Result<Vec<Box<dyn Executor>>, AppError>>()
        ?;
    let failures: Vec<Box<dyn Failure>> = sd.failures
        .iter()
        .map(sytter_failure_table_deserialize)
        .collect::<Result<Vec<Box<dyn Failure>>, AppError>>()
        ?;
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
        &read_to_string(path)
            .map_err(AppError::SytterReadError)
            ?
    )
        .map_err(AppError::SytterDeserializeError)
        ?;
    sytter_deserialize(sd)
        .inspect(|s| debug!("Deserialized Sytter: {:?}", s))
}

impl Sytter {

    pub async fn start(mut self) -> Result<(), AppError> {
        let (_send_to_trigger, receive_from_sytter) = sync_channel(0);
        let (send_to_sytter, receive_from_trigger) = sync_channel(0);
        let trigger: &mut Box<dyn Trigger> = self
            .triggers
            .get_mut(0)
            .ok_or(AppError::SytterMissingComponentError("No triggers!".to_string()))?;

        let res: Result<(), AppError> = trigger
            .trigger_await(
                send_to_sytter,
                receive_from_sytter,
            )
            .await
            .inspect_err(|e| {
                error!("Error in watch loop: {:?}", e);
            })
            .and_then(|()| {
                loop {
                    info!("Waiting for message from trigger...");
                    let trigger_message = receive_from_trigger.recv();
                    debug!("Got trigger message: {:?}", trigger_message);
                    let _ = self
                        .conditions
                        .get(0)
                        .ok_or(AppError::SytterMissingComponentError("No conditions!".to_string()))?
                        .check_condition()
                        .and_then(|cond| {
                            if cond {
                                debug!("Conditional is true, executing...");
                                self.executors
                                    .get(0)
                                    .ok_or(AppError::SytterMissingComponentError("No executors!".to_string()))?
                                    .execute()
                                    .inspect(|_| { info!("Execution successful.") })
                            } else {
                                debug!("Conditional is false.");
                                // Log that the condition fell through.
                                Ok(())
                            }
                        })
                        .or_else(|e| {
                            error!("Executor failed: {:?}", e);
                            self
                                .failures
                                .get(0)
                                .ok_or(AppError::SytterMissingComponentError("No failures!".to_string()))?
                                .execute(e)
                                .inspect(|_| {
                                    info!("Failure handler successful!");
                                })
                        })
                        .map(|_| { () } )
                        .inspect_err(|e| {
                            error!("It blowed up bad in condition! {:?}", e)
                        });
                };
            });
        match res {
            Ok(_) => (),
            Err(e) => error!("It blowed up bad in trigger! {:?}", e),
        }
        Ok(())
    }
}
