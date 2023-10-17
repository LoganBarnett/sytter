use serde::Deserialize;

use crate::{
    condition::Condition,
    executor::Executor,
    failure::Failure, error::AppError, shell::{shell_exec_check, shell_exec_outputs},
};

fn shell_default_exit_codes() -> Vec<i32> {
    vec!(0)
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

impl Condition for ShellCondition {
    fn check_condition(&self) -> Result<bool, AppError> {
        shell_exec_check(
            &self.shell,
            &self.expected_exit_codes,
            &self.script,
        )
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

impl Executor for ShellExecutor {
    fn execute(&self) -> Result<(), AppError> {
        shell_exec_outputs(&self.shell, &self.script)
            .map(|_| { () } )
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ShellFailure {
    #[serde(default = "shell_default_exit_codes")]
    pub expected_exit_codes: Vec<i32>,
    pub script: String,
    pub shell: String,
}

impl Failure for ShellFailure {
    // TODO: This should also take the status.
    fn execute(&self, _error: AppError) -> Result<(), AppError> {
        shell_exec_outputs(&self.shell, &self.script)
            .map(|_| ())
    }
}
