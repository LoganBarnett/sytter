use crate::{config::Config, error::AppError};
use core::fmt::Debug;
use dyn_clone::DynClone;

#[typetag::serde(tag = "type")]
pub trait Executor:
    Debug + Sync + Send + DynClone
{
    fn execute(&self, config: &Config) -> Result<(), AppError>;
}

dyn_clone::clone_trait_object!(Executor);
