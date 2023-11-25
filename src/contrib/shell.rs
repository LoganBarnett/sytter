use log::{error, trace};
use serde::Deserialize;
use toml::Table;

use crate::{
    condition::Condition,
    deserialize::vec_i32_des,
    error::AppError,
    executor::Executor,
    failure::Failure,
    shell::{shell_exec_check, shell_exec_outputs},
};

fn shell_default_exit_codes() -> Vec<i32> {
    vec![0]
}

fn shell_default_shell() -> String {
    "/bin/bash".to_string()
}

#[derive(Clone, Debug, Deserialize)]
pub struct ShellCondition {
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

impl Condition for ShellCondition {
    fn check_condition(&self) -> Result<bool, AppError> {
        shell_exec_check(&self.shell, &self.expected_exit_codes, &self.script)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ShellExecutor {
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

impl Executor for ShellExecutor {
    fn execute(&self) -> Result<(), AppError> {
        shell_exec_outputs(&self.shell, &self.script)
            .inspect(|_| trace!("Executed '{:?}' successfully.", self.script))
            .inspect_err(|_| error!("Execution of '{:?}' failed!", self.script))
            .map(|_| ())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ShellFailure {
    #[serde(default = "shell_default_exit_codes")]
    pub expected_exit_codes: Vec<i32>,
    pub script: String,
    pub shell: String,
}

pub fn shell_failure_toml_deserialize(
    section_data: &Table,
) -> Result<Box<dyn Failure>, AppError> {
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
            .unwrap_or("/bin/bash".to_string()),
    }))
}

impl Failure for ShellFailure {
    // TODO: This should also take the status.
    fn execute(&self, _error: AppError) -> Result<(), AppError> {
        shell_exec_outputs(&self.shell, &self.script).map(|_| ())
    }
}
