use crate::{
  error::AppError,
  trigger::Trigger,
  macos::device::device_connection_listen_start,
};
use log::*;
use serde::{Deserialize, Serialize};
use std::any::type_name;
use std::str::FromStr;
use std::sync::mpsc::{Receiver, SyncSender};
use strum_macros::EnumString;
use toml::Table;

#[derive(Clone, Debug, Deserialize, EnumString, PartialEq, Serialize)]
pub enum DeviceConnectionEvent {
  Add,
  Remove,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeviceConnectionTrigger {
  pub events: Vec<DeviceConnectionEvent>,
}

pub fn device_connection_event_deserialize(
  s: String,
) -> Result<DeviceConnectionEvent, AppError> {
  // For whatever reason, Rust can't make up its mind here.  Either the error
  // is a ParseError or it is Infallible.  Either one selected will be an
  // error to the other.  I don't know how to get actual meaningful error
  // messages out of this, and I strongly suspect a bug in Rust itself.
  DeviceConnectionEvent::from_str(&s)
    .map_err(|_e| AppError::DeviceConnectionEventParseError())
}

pub fn device_connection_toml_deserialize(
  section_data: &Table,
) -> Result<Box<dyn Trigger>, AppError> {
  Ok(Box::new(DeviceConnectionTrigger {
    events:
      section_data
        .get("events")
        .ok_or(AppError::DeviceConnectionEventsMissingError())
        .and_then(|candidate| {
          candidate
            .as_array()
            .ok_or(AppError::DeviceConnectionEventsParseError())
        })
        .and_then(|vec| {
          vec
            .into_iter()
            .map(|s| {
              s
                .as_str()
                .ok_or(AppError::DeviceConnectionEventParseError())
                .map(|s| s.to_owned())
                .and_then(device_connection_event_deserialize)
            })
            .collect::<Result<Vec<DeviceConnectionEvent>, AppError>>()
        })?
  }))
}

#[typetag::serde]
impl Trigger for DeviceConnectionTrigger {

  fn trigger_await(
    &mut self,
    send_to_sytter: SyncSender<String>,
    _receive_from_sytter: Receiver<String>,
  ) -> Result<(), AppError> {
    let send_to_sytter_threaded = send_to_sytter.clone();
    if cfg!(target_os = "macos") {
      info!("Listening for device connection events {:?}", self.events);
      let events = self.events.clone();
      // TODO: Connect plumbing such that any trigger can be shut down via
      // a cleanup function that is returned when listening.
      let _cleanup_fn =
        device_connection_listen_start(Box::new(move |p: DeviceConnectionEvent| {
          trace!(
            "Signaling sytter from {} for event {:?}.",
            type_name::<DeviceConnectionTrigger>(),
            p,
          );
          if events.contains(&p) {
            match send_to_sytter_threaded.send("foo".to_string()) {
              Ok(_) => trace!(
                "Signal to sytter from {} successful!",
                type_name::<DeviceConnectionTrigger>(),
              ),
              Err(e) => trace!(
                "Error triggering sytter from {}: {:?}",
                type_name::<DeviceConnectionTrigger>(),
                e,
              ),
            };
          } else {
            trace!(
              "{} {:?} skipped because it is not in {:?}",
              type_name::<DeviceConnectionTrigger>(),
              p,
              events,
            );
          }
        }))?;
      trace!("Setup listener for device connection events.");
      // None of this is called at the right time. trigger_await is async so we
      // can make the setup happen on its own thread. In the case of
      // macos/device.rs we already have a thread. I'm not sure if that overhead
      // is necessary but whatever.
      // This is the event to clean up. Just block on it.
      // let _ = receive_from_sytter.recv();
      // debug!("Sytter is closing. Cleaning up power hooks...");
      // cleanup_fn()?;
      // debug!("Device connection listener cleanup done!");
    } else {
      error!("OS not supported for device connection events!");
    }
    Ok(())
  }

}
