use crate::error::AppError;
use core::fmt::Debug;

pub trait Executor: Debug + Sync + Send + serde_traitobject::Deserialize {
    fn execute(&self) -> Result<(), AppError>;
}
