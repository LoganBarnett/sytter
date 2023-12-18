use crate::{config::Config, error::AppError};
use core::fmt::Debug;
use dyn_clone::DynClone;

#[typetag::serde(tag = "type")]
pub trait Failure:
    Debug + Sync + Send + DynClone
{
    fn execute(&self, config: &Config, error: AppError) -> Result<(), AppError>;
}

dyn_clone::clone_trait_object!(Failure);
