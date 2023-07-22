use crate::error::AppError;

pub trait Condition {
   fn check_condition(&self) -> Result<bool, AppError>;
}
