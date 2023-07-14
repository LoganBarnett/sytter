use crate::error::AppError;

pub trait Executor {
    fn execute(&self) -> Result<(), AppError>;
}
