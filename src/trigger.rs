use crate::error::AppError;
use async_trait::async_trait;
use core::fmt::Debug;
use std::sync::mpsc::{Receiver, SyncSender};

#[async_trait]
pub trait Trigger:
    Debug + Sync + Send + serde_traitobject::Deserialize
{
    async fn trigger_await(
        &mut self,
        send_to_sytter: SyncSender<String>,
        receive_from_sytter: Receiver<String>,
    ) -> Result<(), AppError>;
}
