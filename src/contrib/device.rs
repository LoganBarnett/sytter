use crate::{error::AppError, trigger::Trigger};

#[cfg(target_os = "macos")]
use crate::macos::device::device_connection_listen_start;
use serde::{Deserialize, Serialize};
use std::any::type_name;
use std::str::FromStr;
use std::sync::mpsc::{Receiver, SyncSender};
use strum_macros::EnumString;
use toml::Table;
use tracing::*;

#[derive(Clone, Debug, Deserialize, EnumString, PartialEq, Serialize)]
pub enum DeviceConnectionEvent {
  Add,
  Remove,
}

/// Platform-agnostic device types
#[derive(Clone, Debug, Deserialize, EnumString, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum DeviceType {
  Any,
  Usb,
  Storage,
  Network,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeviceConnectionTrigger {
  pub events: Vec<DeviceConnectionEvent>,
  #[serde(default)]
  pub device_types: Vec<DeviceType>,

  // macOS-specific overrides (if specified, device_types is ignored)
  #[serde(default)]
  pub macos_device_class: Option<String>,
  #[serde(default)]
  pub macos_notification_type: Option<String>,
}

impl DeviceConnectionTrigger {
  /// Expand device_types list, handling "any" → all concrete types
  fn expand_device_types(&self) -> Vec<DeviceType> {
    if self.device_types.contains(&DeviceType::Any) {
      vec![DeviceType::Usb, DeviceType::Storage, DeviceType::Network]
    } else {
      self.device_types.clone()
    }
  }

  /// Get the device classes to monitor, preferring macOS override, then mapping from device_types
  #[cfg(target_os = "macos")]
  fn get_device_classes(&self) -> Vec<String> {
    // If macOS override is specified, use only that
    if let Some(ref class) = self.macos_device_class {
      return vec![class.clone()];
    }

    // Otherwise, expand device_types and map to macOS classes
    self
      .expand_device_types()
      .iter()
      .map(Self::map_device_type_to_macos)
      .collect()
  }

  /// Get the notification type to use, preferring macOS override, then default
  #[cfg(target_os = "macos")]
  fn get_notification_type(&self) -> String {
    self
      .macos_notification_type
      .as_ref()
      .cloned()
      .unwrap_or_else(|| "FirstMatch".to_string())
  }

  /// Map platform-agnostic device types to macOS IOKit device classes
  #[cfg(target_os = "macos")]
  fn map_device_type_to_macos(device_type: &DeviceType) -> String {
    match device_type {
      DeviceType::Any => panic!("Any should be expanded before mapping"),
      DeviceType::Usb => "IOUSBHostDevice".to_string(),
      DeviceType::Storage => "IOMedia".to_string(),
      DeviceType::Network => "IONetworkInterface".to_string(),
    }
  }
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
  let trigger: DeviceConnectionTrigger =
    section_data.clone().try_into().map_err(|e| {
      AppError::SytterDeserializeRawError(format!(
        "Failed to deserialize device connection trigger: {:?}",
        e
      ))
    })?;

  // Validate that we have either device_types or macos_device_class
  if trigger.device_types.is_empty() && trigger.macos_device_class.is_none() {
    return Err(AppError::SytterDeserializeRawError(
      "Either 'device_types' or 'macos_device_class' must be specified"
        .to_string(),
    ));
  }

  Ok(Box::new(trigger))
}

#[typetag::serde]
impl Trigger for DeviceConnectionTrigger {
  fn trigger_await(
    &mut self,
    send_to_sytter: SyncSender<String>,
    _receive_from_sytter: Receiver<String>,
  ) -> Result<(), AppError> {
    if cfg!(target_os = "macos") {
      let device_classes = self.get_device_classes();
      let notification_type = self.get_notification_type();

      info!(
        "Device connection trigger starting. Events: {:?}, Device types: {:?}, macOS classes: {:?}, macOS notification: {}",
        self.events, self.device_types, device_classes, notification_type
      );

      let events = self.events.clone();

      if events.is_empty() {
        warn!("No events specified for device connection trigger");
      }

      // Register a listener for each device class
      let mut _cleanup_fns = Vec::new();

      for device_class in device_classes {
        let send_to_sytter_for_this_listener = send_to_sytter.clone();
        let events_for_this_listener = events.clone();
        let device_class_name = device_class.clone();

        // TODO: Connect plumbing such that any trigger can be shut down via
        // a cleanup function that is returned when listening.
        debug!(
          "Calling device_connection_listen_start for device class: {}...",
          device_class
        );
        let cleanup_fn = device_connection_listen_start(
          &device_class,
          &notification_type,
          Box::new(move |p: DeviceConnectionEvent| {
            debug!(
              "Device event received for {}: {:?} (trigger watching: {:?})",
              device_class_name,
              p,
              events_for_this_listener,
            );
            if events_for_this_listener.contains(&p) {
              info!(
                "Device event {:?} for {} matches trigger, signaling sytter",
                p,
                device_class_name
              );
              match send_to_sytter_for_this_listener.send("foo".to_string()) {
                Ok(_) => info!(
                  "Successfully signaled sytter for device event {:?} ({})",
                  p,
                  device_class_name
                ),
                Err(e) => error!(
                  "Failed to signal sytter for device event {:?} ({}): {:?}",
                  p,
                  device_class_name,
                  e,
                ),
              };
            } else {
              debug!(
                "Device event {:?} for {} does not match trigger (expecting {:?}), skipping",
                p,
                device_class_name,
                events_for_this_listener,
              );
            }
          }),
        )
        .inspect(|_| info!("Device connection listener successfully registered for {}", device_class))
        .inspect_err(|e| error!("Failed to register device connection listener for {}: {:?}", device_class, e))?;

        _cleanup_fns.push(cleanup_fn);
      }

      info!("Device connection trigger is now active with {} listeners waiting for events", _cleanup_fns.len());
      // None of this is called at the right time. trigger_await is async so we
      // can make the setup happen on its own thread. In the case of
      // macos/device.rs we already have a thread. I'm not sure if that overhead
      // is necessary but whatever.
      // This is the event to clean up. Just block on it.
      // let _ = receive_from_sytter.recv();
      // debug!("Sytter is closing. Cleaning up power hooks...");
      // for cleanup_fn in cleanup_fns {
      //   cleanup_fn()?;
      // }
      // debug!("Device connection listener cleanup done!");
    } else {
      error!("OS not supported for device connection events!");
    }
    Ok(())
  }
}
