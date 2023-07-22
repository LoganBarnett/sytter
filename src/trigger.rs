use async_trait::async_trait;
use crate::error::AppError;
use std::sync::mpsc::{SyncSender, Receiver};

#[async_trait]
pub trait Trigger {

    fn trigger_await(
        &mut self,
        send_to_sytter: SyncSender<String>,
        receive_from_sytter: Receiver<String>,
    ) -> Result<(), AppError>;

}
