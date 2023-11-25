use crate::error::AppError;
use core::fmt::Debug;

pub trait Condition:
    Debug + Sync + Send + serde_traitobject::Deserialize
{
    fn check_condition(&self) -> Result<bool, AppError>;
}
