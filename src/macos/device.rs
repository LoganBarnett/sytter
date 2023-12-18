#![cfg(target_os = "macos")]
/**
 * https://developer.apple.com/documentation/iokit/1514362-ioserviceaddmatchingnotification
 * This example looks very complete:
 * https://github.com/opensource-apple/IOUSBFamily/blob/master/Examples/Another%20USB%20Notification%20Example/USBNotificationExample.c
 */
use crate::{
  contrib::device::{
    DeviceConnectionEvent,
  },
  error::AppError,
  macos::{
    event::{
      listen_start,
      CallbackData,
      ListenResult,
    },
    // native::dict_set_i32,
  },
};
use sytter_macos_bindings::{
  // HRESULT,
  // IOCFPlugInInterface,
  IONotificationPortRef,
  IOServiceAddMatchingNotification,
  IOServiceMatching,
  io_object_t,
  // io_service_t,
  io_iterator_t,
  kIOFirstMatchNotification,
  kIOUSBDeviceClassName,
  // kUSBProductID,
  // kUSBVendorID,
  // kern_return_t,
  // NSDictionary,
};
use log::*;
use std::{
  // any::Any,
  ffi::c_void,
};

impl CallbackData for DeviceConnectionEvent {
  // fn as_any(&self) -> &dyn Any {
  //   self
  // }
}

// extern "C" fn device_connection_callback(
//   kr: kern_return_t,
//   device: io_service_t,
//   plugin_interface: IOCFPlugInInterface,
//   score: i32,
//   res: HRESULT,
// ) -> () {
//   debug!("USB device connection detected where it shouldn't work!");
// }

// TODO: Ensure we look over the documentation here to verify the type
// signature.  The commented signature above was expected, but from what I need
// to verify.  It is not accepted by the currently generated bindings.
extern "C" fn device_connection_callback(
  _refcon: *mut c_void,
  _notifier: io_iterator_t,
) -> () {
  debug!("USB device connection detected!");
}

pub fn device_connection_listen_start(
  callback: impl FnMut(DeviceConnectionEvent) -> () + Send + Sync + 'static,
) -> Result<Box<dyn FnOnce() -> Result<(), AppError>>, AppError> {
  let _usb_vendor: i32 = 0;
  let _usb_product: i32 = 0;
  // TODO: Check if we actually got one back.  The value will be 0 if not.
  let matching = unsafe {
    IOServiceMatching(kIOUSBDeviceClassName.as_ptr() as *mut i8)
  };
  unsafe {
    trace!("Found service: {:?}", *matching)
  };
  // dict_set_i32(
  //   matching,
  //   kUSBVendorID.as_ptr() as *mut c_void,
  //   usb_vendor,
  // );
  // dict_set_i32(
  //   matching,
  //   kUSBProductID.as_ptr() as *mut c_void,
  //   usb_product,
  // );
  let matching_for_listen = matching.clone();
  let stop_callback = listen_start(
    Box::new(
      move |notifier: &mut io_object_t, port_ref: &mut IONotificationPortRef, refcon: &mut usize|
        unsafe {
          ListenResult::kern_return_t(IOServiceAddMatchingNotification(
            *port_ref,
            // Some fixed and available filter.
            kIOFirstMatchNotification.as_ptr() as *mut i8,
            matching_for_listen,
            Option::Some(device_connection_callback),
            // This is tossed back to the callback, so we know which instance of
            // the callback was invoked.
            *refcon as *mut c_void,
            // This is an io_object_t but an io_iterator_t is expected.  Why
            // does it work?  Probably because they are backed by the same
            // underlying type.
            notifier,
          ))
        }
    ),
    callback,
  )?;
  Ok(Box::new(move || {
    let (notifier, port_ref) = stop_callback();
    device_connect_listen_stop(notifier, port_ref)
  }))
}

pub fn device_connect_listen_stop(
  mut _notifier: io_object_t,
  _port_ref: IONotificationPortRef,
) -> Result<(), AppError> {
  // TODO: Find the inverse of IOServiceAddMatchingNotification.
  Ok(())
}
