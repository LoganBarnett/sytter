use dyn_clone::DynClone;

use crate::{config::Config, error::AppError};
use core::fmt::Debug;

#[typetag::serde(tag = "type")]
pub trait Condition:
    Debug + Sync + Send + DynClone
{
    fn check_condition(&self, config: &Config) -> Result<bool, AppError>;
}

dyn_clone::clone_trait_object!(Condition);
