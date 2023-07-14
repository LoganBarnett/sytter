use crate::{
    condition::Condition,
    executor::Executor,
    failure::Failure,
};

pub struct ShellCondition {
    pub shell: String,
    pub expected_exit_codes: Vec<usize>,
}

impl Condition for ShellCondition {
    fn check_condition(&self) -> bool {
        todo!()
    }
}

pub struct ShellExecutor {
    pub shell: String,
}

impl Executor for ShellExecutor {
    fn execute(&self) -> Result<(), crate::error::AppError> {
        todo!()
    }
}

pub struct ShellFailure {
    pub shell: String,
}

impl Failure for ShellFailure {
    fn execute(&self) -> Result<(), crate::error::AppError> {
        todo!()
    }
}
