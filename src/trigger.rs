use crate::error::AppError;
use core::fmt::Debug;
use std::sync::mpsc::{Receiver, SyncSender};

#[typetag::serde(tag = "type")]
pub trait Trigger:
  Debug + Sync + Send
{
  fn trigger_await(
    &mut self,
    send_to_sytter: SyncSender<String>,
    receive_from_sytter: Receiver<String>,
  ) -> Result<(), AppError>;
}
