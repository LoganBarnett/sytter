use crate::state::{State, SytterVariable};
use crate::{
    error::AppError,
    trigger::Trigger,
    macos::power::sleep_listen_start,
};
use log::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::mpsc::{Receiver, SyncSender};
use strum_macros::{Display, EnumString};
use toml::Table;

#[derive(Clone, Debug, Deserialize, Display, EnumString, PartialEq, Serialize)]
pub enum PowerEvent {
    Boot,     // Not supported.
    Shutdown, // Not supported.
    Sleep,
    Wake,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PowerTrigger {
    pub events: Vec<PowerEvent>,
}

pub fn power_event_deserialize(s: String) -> Result<PowerEvent, AppError> {
    // For whatever reason, Rust can't make up its mind here.  Either the error
    // is a ParseError or it is Infallible.  Either one selected will be an
    // error to the other.  I don't know how to get actual meaningful error
    // messages out of this, and I strongly suspect a bug in Rust itself.
    PowerEvent::from_str(&s).map_err(|_e| AppError::PowerEventParseError)
}

pub fn power_trigger_toml_deserialize(
    section_data: &Table,
) -> Result<Box<dyn Trigger>, AppError> {
    Ok(Box::new(PowerTrigger {
        events:
            section_data
                .get("events")
                .ok_or(AppError::PowerEventsMissingError)
                .and_then(|candidate| {
                    candidate
                        .as_array()
                        .ok_or(AppError::PowerEventParseError)
                })
                .and_then(|vec| {
                    vec
                        .into_iter()
                        .map(|s| {
                            s
                                .as_str()
                                .ok_or(AppError::PowerEventParseError)
                                .map(|s| s.to_owned())
                                .and_then(power_event_deserialize)
                        })
                        .collect::<Result<Vec<PowerEvent>, AppError>>()
                })?
    }))
}

#[typetag::serde]
impl Trigger for PowerTrigger {
  fn trigger_await(
    &mut self,
    send_to_sytter: SyncSender<String>,
    _receive_from_sytter: Receiver<String>,
  ) -> Result<(), AppError> {
    let send_to_sytter_threaded = send_to_sytter.clone();
    if cfg!(target_os = "macos") {
      info!("Listening for power event {:?}", self.events);
      let events = self.events.clone();
      // TODO: Connect plumbing such that any trigger can be shut down via
      // a cleanup function that is returned when listening.
      let _cleanup_fn =
        sleep_listen_start(Box::new(move |p: PowerEvent| {
          trace!(
            "Signaling sytter from PowerTrigger for event {:?}.",
            p,
          );
          if events.contains(&p) {
            State::set_variable(SytterVariable {
              key: "sytter_power_event".into(),
              value: p.to_string(),
            });
            match send_to_sytter_threaded.send("foo".to_string()) {
              Ok(_) => trace!("Signal to sytter from PowerTrigger successful!"),
              Err(e) => trace!(
                "Error triggering sytter from PowerTrigger: {:?}",
                e
              ),
            };
          } else {
            trace!(
              "Power event {:?} skipped because it is not in {:?}",
              p,
              events,
            );
          }
        }))?;
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
