use crate::{
    condition::Condition,
    executor::Executor,
    failure::Failure, error::AppError,
};

#[derive(Clone)]
pub struct ShellCondition {
    pub shell: String,
    pub expected_exit_codes: Vec<usize>,
}

impl Condition for ShellCondition {
    fn check_condition(&self) -> Result<bool, AppError> {
        todo!()
    }
}

#[derive(Clone)]
pub struct ShellExecutor {
    pub shell: String,
}

impl Executor for ShellExecutor {
    fn execute(&self) -> Result<(), AppError> {
        todo!()
    }
}

#[derive(Clone)]
pub struct ShellFailure {
    pub shell: String,
}

impl Failure for ShellFailure {
    fn execute(&self, error: AppError) -> Result<(), AppError> {
        todo!()
    }
}
