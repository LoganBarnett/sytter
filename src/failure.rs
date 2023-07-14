use crate::{error::AppError};

pub trait Failure {
    fn execute(&self) -> Result<(), AppError>;
}
