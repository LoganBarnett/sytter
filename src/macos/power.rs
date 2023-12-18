#![cfg(target_os = "macos")]

use num::FromPrimitive;
use crate::{
  error::AppError,
  macos::event::{
    refcon_callback,
    listen_start,
    CallbackData,
    ListenResult,
  },
  contrib::power::PowerEvent,
};
use sytter_macos_bindings::{
  AppleIoMessage,
  AppleIoReturn,
  IOAllowPowerChange,
  IODeregisterForSystemPower,
  IONotificationPortDestroy,
  IONotificationPortRef,
  IORegisterForSystemPower,
  UInt32,
  io_connect_t,
  io_object_t,
  io_service_t,
  kIOMessageCanSystemSleep,
  kIOMessageSystemHasPoweredOn,
  kIOMessageSystemWillNotSleep,
  kIOMessageSystemWillPowerOn,
  kIOMessageSystemWillSleep,
  kIOReturnSuccess,
};
// use std::any::Any;
/**
 * For Darwin (macOS), see
 * https://developer.apple.com/documentation/iokit/1557114-ioregisterforsystempower
 * for some relevant documentation.
 * This also appears to be an information trove:
 * https://docs.darlinghq.org/internals/macos-specifics/mach-ports.html
 */
// https://developer.apple.com/documentation/iokit/ioserviceinterestcallback
// io_connect_t IORegisterForSystemPower(void *refcon, IONotificationPortRef *thePortRef, IOServiceInterestCallback callback, io_object_t *notifier);
use std::ffi::c_void;
// Gives us macros such as debug! and error! See logging.rs for setup.
use log::*;

impl CallbackData for PowerEvent {
  // fn as_any(&self) -> &dyn Any {
  //   self
  // }
}

extern "C" fn power_change_callback(
  refcon: *mut c_void,
  _service: io_service_t,
  message_type: UInt32,
  message_argument: *mut c_void,
) -> () {
  // We cheated.  Instead of sending a pointer, we just sent a number and we get
  // it back.
  let refcon_value = refcon as usize;
  debug!(
    "Got macOS power event! {:?}",
    AppleIoMessage::from_u32(message_type as u32).unwrap(),
  );
  // These two messages require acknowledgement or they will stall the
  // sleeping process.
  if message_type == kIOMessageCanSystemSleep
    || message_type == kIOMessageSystemWillSleep
  {
    let power_event = Box::new(if message_type == kIOMessageCanSystemSleep {
      debug!("Got message kIOMessageCanSystemSleep.");
      PowerEvent::Unknown
    } else {
      debug!("Got message kIOMessageWillSystemSleep.");
      PowerEvent::Sleep
    });
    // Be wary about what's passed as the power event.  A type mismatch won't be
    // caught until runtime.
    let kernel_port: io_connect_t = refcon_callback(refcon_value, power_event)
      .unwrap();
    trace!("Transmuted refcon to our closure.");
    // According to documentation here:
    // https://developer.apple.com/library/archive/qa/qa1340/_index.html
    // The notification ID and the message argument are the same.
    debug!("message_argument: {:?}", message_argument);
    debug!(
      "Notifying OS that we allow the power change with notify_id {:?}...",
      message_argument,
      // notify_id,
    );
    // IOReturn/kern_return_t are i32 but the codes to compare are u32.  An
    // issue with binding generation? Or just bad headers?
    let io_return =
      unsafe { IOAllowPowerChange(kernel_port, message_argument) };
    if io_return as u32 == kIOReturnSuccess {
      debug!("Power change notification successful!");
    } else {
      error!(
            "Error ({:?}) allowing power change notification. \
is will keep the machine from sleeping for 30+ seconds. \
ing kernel_port {:?} and notifiy_id {:?}",
        // TODO: unwrap or cast to an unknown type.
        AppleIoReturn::from_u32(io_return as u32).unwrap(),
        kernel_port,
        message_argument,
      );
    }
  }
  // kIOMessageSystemWillPowerOn is before devices have power.  We don't want
  // to run at this point as it will block or fail.
  else if message_type == kIOMessageSystemWillPowerOn {
      debug!("Got message kIOMessageSystemWillPowerOn.");
  }
  // kIOMessageSystemHasPoweredOn runs after devices have gotten power.  This
  // is where we want to do our work.
  else if message_type == kIOMessageSystemHasPoweredOn {
      debug!("Got message kIOMessageSystemHasPoweredOn.");
    refcon_callback(refcon_value, Box::new(PowerEvent::Wake)).unwrap();
      // closure(Box::new(PowerEvent::Wake));
  }
  // These should be no action - someone vetoed sleep.
  else if message_type == kIOMessageSystemWillNotSleep {
      debug!("Got message kIOMessageSystemWillNotSleep.");
  }
}

#[cfg(target_os = "macos")]
pub fn sleep_listen_start(
    callback: impl FnMut(PowerEvent) -> () + Send + Sync + 'static,
) -> Result<Box<dyn FnOnce() -> Result<(), AppError>>, AppError> {
  let stop_callback = listen_start(
    Box::new(
      |notifier: &mut io_object_t, port_ref: &mut IONotificationPortRef, refcon: &mut usize|
        unsafe {
          trace!("port_ref in delegate: {:x?}", port_ref);
          ListenResult::io_connect_t(IORegisterForSystemPower(
            // This is tossed back to the callback, so we know which instance of
            // the callback was invoked.
            *refcon as *mut c_void,
            &mut *port_ref,
            Option::Some(power_change_callback),
            &mut *notifier,
          ))
        }
    ),
    callback,
  )?;
  // Is there any harm in just returning the notifier and port_ref directly?
  Ok(Box::new(move || {
    let (notifier, port_ref) = stop_callback();
    sleep_listen_stop(notifier, port_ref)
  }))
}

// This is kind of a pain to pass the information around.  We could simply
// return a de-register function upon listening, which is just this.
#[cfg(target_os = "macos")]
pub fn sleep_listen_stop(
  mut notifier: io_object_t,
  port_ref: IONotificationPortRef,
) -> Result<(), AppError> {
  trace!("Cleaning up macOS power listener...");
  let io_return = unsafe { IODeregisterForSystemPower(&mut notifier) };
  // See this for interpreting the kernel_return_t/IOReturn value:
  // https://gist.github.com/MLKrisJohnson/eb5e1cb623694372676c938be82c9bb4
  // It is not documented elsewhere.  I should be able just look for 0.
  if io_return == 0 {
    Err(AppError::PowerHookRegistrationFailed)
  } else {
    unsafe { IONotificationPortDestroy(port_ref) };
    trace!("Power listening events for macOS cleaned up.");
    Ok(())
  }
}
