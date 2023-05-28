use crate::error::AppError;

pub trait Condition {
   fn check_condition(&self) -> bool;
}
