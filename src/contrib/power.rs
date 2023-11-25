use crate::{error::AppError, trigger::Trigger};
use async_trait::async_trait;
use log::*;
use serde::Deserialize;
use std::sync::mpsc::{Receiver, SyncSender};
use toml::Table;

#[derive(Clone, Debug, Deserialize)]
pub enum PowerEvent {
    Boot,     // Not supported.
    Shutdown, // Not supported.
    Sleep,
    Wake,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PowerTrigger {
    pub event: PowerEvent,
}

pub fn power_trigger_toml_deserialize(
    _section_data: &Table,
) -> Result<Box<dyn Trigger>, AppError> {
    Ok(Box::new(PowerTrigger {
        event: PowerEvent::Sleep,
    }))
}

#[async_trait]
impl Trigger for PowerTrigger {
    async fn trigger_await(
        &mut self,
        send_to_sytter: SyncSender<String>,
        _receive_from_sytter: Receiver<String>,
    ) -> Result<(), AppError> {
        let send_to_sytter_threaded = send_to_sytter.clone();
        if cfg!(target_os = "macos") {
            info!("Listening for power event {:?}", self.event);
            // TODO: Connect plumbing such that any trigger can be shut down via
            // a cleanup function that is returned when listening.
            let _cleanup_fn =
                crate::power_macos::sleep_listen_start(move || {
                    trace!("Signaling sytter from PowerTrigger.");
                    match send_to_sytter_threaded.send("foo".to_string()) {
                        Ok(_) => trace!(
                            "Signal to sytter from PowerTrigger successful!"
                        ),
                        Err(e) => trace!(
                            "Error trigging sytter from PowerTrigger: {:?}",
                            e
                        ),
                    };
                })?;
            trace!("Setup listener for power events.");
            // None of this is called at the right time. trigger_await is async
            // so we can make the setup happen on its own thread. In the case of
            // power_macos we already have a thread. I'm not sure if that
            // overhead is necessary but whatever.
            // This is the event to clean up. Just block on it.
            // let _ = receive_from_sytter.recv();
            // debug!("Sytter is closing. Cleaning up power hooks...");
            // cleanup_fn()?;
            // debug!("Power cleanup done!");
        } else {
            error!("OS not supported for power events!");
        }
        Ok(())
    }
}
