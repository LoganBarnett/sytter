use log::{error, trace};
use serde::{Deserialize, Serialize};
use toml::Table;
use uuid::Uuid;

use crate::{
  condition::Condition,
  config::Config,
  deserialize::vec_i32_des,
  error::AppError,
  executor::Executor,
  failure::Failure,
  shell::{shell_exec_check, shell_exec_outputs, with_shell_functions},
};

fn shell_default_exit_codes() -> Vec<i32> {
    vec![0]
}

fn shell_default_shell() -> String {
  "/bin/bash".into()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShellCondition {
    pub id: String,
    #[serde(default = "shell_default_exit_codes")]
    pub expected_exit_codes: Vec<i32>,
    pub script: String,
    #[serde(default = "shell_default_shell")]
    pub shell: String,
}

pub fn shell_condition_toml_deserialize(
    section_data: &Table,
) -> Result<Box<dyn Condition>, AppError> {
    Ok(Box::new(ShellCondition {
        id: Uuid::new_v4().to_string(),
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
            .unwrap_or("/bin/bash".to_string()),
        expected_exit_codes: vec_i32_des(
            section_data.get("expected_exit_codes"),
        ),
    }))
}

#[typetag::serde]
impl Condition for ShellCondition {
  fn check_condition(&self, config: &Config) -> Result<bool, AppError> {
    shell_exec_check(
      config.http_port,
      &self.shell,
      &self.expected_exit_codes,
      &with_shell_functions(&self.script),
      &self.id,
    )
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShellExecutor {
    pub id: String,
    #[serde(default = "shell_default_exit_codes")]
    pub expected_exit_codes: Vec<i32>,
    pub script: String,
    #[serde(default = "shell_default_shell")]
    pub shell: String,
}

pub fn shell_executor_toml_deserialize(
    section_data: &Table,
) -> Result<Box<dyn Executor>, AppError> {
    Ok(Box::new(ShellExecutor {
        id: Uuid::new_v4().to_string(),
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
            .unwrap_or("/bin/bash".to_string()),
    }))
}

#[typetag::serde]
impl Executor for ShellExecutor {
  fn execute(&self, config: &Config) -> Result<(), AppError> {
    shell_exec_outputs(
      config.http_port,
      &self.shell,
      &with_shell_functions(&self.script),
      &self.id
    )
      .inspect(|_| trace!("Executed '{:?}' successfully.", self.id))
      .inspect_err(|_| {
        error!("Execution of script failed!  Script:\n{}", self.script);
      })
      .map(|_| ())
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShellFailure {
    pub id: String,
    #[serde(default = "shell_default_exit_codes")]
    pub expected_exit_codes: Vec<i32>,
    pub script: String,
    pub shell: String,
}

pub fn shell_failure_toml_deserialize(
    section_data: &Table,
) -> Result<Box<dyn Failure>, AppError> {
    Ok(Box::new(ShellFailure {
        id: Uuid::new_v4().to_string(),
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
            .unwrap_or("/bin/bash".to_string()),
    }))
}

#[typetag::serde]
impl Failure for ShellFailure {
  // TODO: This should also take the status.
  fn execute(&self, config: &Config, _error: AppError) -> Result<(), AppError> {
    shell_exec_outputs(
      config.http_port,
      &self.shell,
      &with_shell_functions(&self.script),
      &self.id
    ).map(|_| ())
  }
}
