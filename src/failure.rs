use crate::error::AppError;
use core::fmt::Debug;

pub trait Failure: Debug + Sync + Send + serde_traitobject::Deserialize {
    fn execute(&self, error: AppError) -> Result<(), AppError>;
}
