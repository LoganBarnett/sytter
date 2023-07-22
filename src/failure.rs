use crate::{error::AppError};

pub trait Failure {
    fn execute(&self, error: AppError) -> Result<(), AppError>;
}
