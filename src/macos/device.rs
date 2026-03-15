#![cfg(target_os = "macos")]
/**
 * https://developer.apple.com/documentation/iokit/1514362-ioserviceaddmatchingnotification
 * This example looks very complete:
 * https://github.com/opensource-apple/IOUSBFamily/blob/master/Examples/Another%20USB%20Notification%20Example/USBNotificationExample.c
 */
use crate::{
  contrib::device::DeviceConnectionEvent,
  error::AppError,
  macos::event::{listen_start, refcon_callback, CallbackData, ListenResult},
  state::{State, SytterVariable},
};
use std::{
  ffi::{c_void, CString},
  sync::{Arc, Mutex},
};
use sytter_macos_bindings::{
  io_iterator_t, io_object_t, kCFAllocatorDefault, kCFNumberSInt32Type,
  kIOFirstMatchNotification, kIOFirstPublishNotification, CFNumberGetValue,
  CFNumberRef, CFRelease, CFStringBuiltInEncodings_kCFStringEncodingUTF8,
  CFStringGetCString, CFStringRef, CFTypeRef, IOIteratorNext,
  IONotificationPortRef, IOObjectRelease, IORegistryEntryCreateCFProperty,
  IOServiceAddMatchingNotification, IOServiceMatching,
};
use tracing::*;

impl CallbackData for DeviceConnectionEvent {}

/// Helper function to convert CFStringRef to Rust String
unsafe fn cfstring_to_string(cf_string: CFStringRef) -> Option<String> {
  if cf_string.is_null() {
    return None;
  }

  let mut buffer = vec![0u8; 256];
  let success = CFStringGetCString(
    cf_string,
    buffer.as_mut_ptr() as *mut i8,
    buffer.len() as i64,
    CFStringBuiltInEncodings_kCFStringEncodingUTF8,
  );

  if success != 0 {
    // Find the null terminator
    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    String::from_utf8(buffer[..len].to_vec()).ok()
  } else {
    None
  }
}

/// Helper function to convert CFNumberRef to i32
unsafe fn cfnumber_to_i32(cf_number: CFNumberRef) -> Option<i32> {
  if cf_number.is_null() {
    return None;
  }

  let mut value: i32 = 0;
  let success = CFNumberGetValue(
    cf_number,
    kCFNumberSInt32Type,
    &mut value as *mut i32 as *mut c_void,
  );

  if success != 0 {
    Some(value)
  } else {
    None
  }
}

/// Extract a string property from an IOKit device
unsafe fn get_device_string_property(
  device: io_object_t,
  key: &str,
) -> Option<String> {
  let key_cstring = CString::new(key).ok()?;
  let key_cfstring = sytter_macos_bindings::CFStringCreateWithCString(
    kCFAllocatorDefault,
    key_cstring.as_ptr(),
    CFStringBuiltInEncodings_kCFStringEncodingUTF8,
  );

  if key_cfstring.is_null() {
    return None;
  }

  let property = IORegistryEntryCreateCFProperty(
    device,
    key_cfstring,
    kCFAllocatorDefault,
    0,
  );

  CFRelease(key_cfstring as CFTypeRef);

  if property.is_null() {
    return None;
  }

  let result = cfstring_to_string(property as CFStringRef);
  CFRelease(property);
  result
}

/// Extract an integer property from an IOKit device
unsafe fn get_device_int_property(
  device: io_object_t,
  key: &str,
) -> Option<i32> {
  let key_cstring = CString::new(key).ok()?;
  let key_cfstring = sytter_macos_bindings::CFStringCreateWithCString(
    kCFAllocatorDefault,
    key_cstring.as_ptr(),
    CFStringBuiltInEncodings_kCFStringEncodingUTF8,
  );

  if key_cfstring.is_null() {
    return None;
  }

  let property = IORegistryEntryCreateCFProperty(
    device,
    key_cfstring,
    kCFAllocatorDefault,
    0,
  );

  CFRelease(key_cfstring as CFTypeRef);

  if property.is_null() {
    return None;
  }

  let result = cfnumber_to_i32(property as CFNumberRef);
  CFRelease(property);
  result
}

/// Extract device properties and set them as state variables
unsafe fn extract_and_set_device_properties(device: io_object_t) {
  // Extract common device properties
  if let Some(name) = get_device_string_property(device, "USB Product Name")
    .or_else(|| get_device_string_property(device, "IOClass"))
  {
    State::set_variable(SytterVariable {
      key: "sytter_device_name".into(),
      value: name,
    });
  }

  if let Some(vendor_id) = get_device_int_property(device, "idVendor") {
    State::set_variable(SytterVariable {
      key: "sytter_device_vendor_id".into(),
      value: format!("0x{:04x}", vendor_id),
    });
  }

  if let Some(product_id) = get_device_int_property(device, "idProduct") {
    State::set_variable(SytterVariable {
      key: "sytter_device_product_id".into(),
      value: format!("0x{:04x}", product_id),
    });
  }

  if let Some(serial) = get_device_string_property(device, "USB Serial Number")
  {
    State::set_variable(SytterVariable {
      key: "sytter_device_serial_number".into(),
      value: serial,
    });
  }

  if let Some(location_id) = get_device_int_property(device, "locationID") {
    State::set_variable(SytterVariable {
      key: "sytter_device_location_id".into(),
      value: format!("0x{:08x}", location_id),
    });
  }

  if let Some(bsd_name) = get_device_string_property(device, "BSD Name") {
    State::set_variable(SytterVariable {
      key: "sytter_device_bsd_name".into(),
      value: bsd_name,
    });
  }
}

// Callback invoked when a device matching our filter is added.
// CRITICAL: We must iterate through the iterator and empty it, or subsequent
// notifications will not fire. See IOServiceAddMatchingNotification docs.
extern "C" fn device_connection_callback(
  refcon: *mut c_void,
  iterator: io_iterator_t,
) -> () {
  if refcon.is_null() {
    error!("Device connection callback received NULL refcon!");
    return;
  }

  let refcon_value = refcon as usize;
  info!(
    "Device connection notification fired. refcon={:x}",
    refcon_value
  );

  if iterator.0 == 0 {
    error!("Device connection callback received NULL iterator!");
    return;
  }

  debug!("Processing device iterator: {:?}", iterator);

  let mut count = 0;
  let max_iterations = 100; // Safety limit to prevent infinite loops
                            // Iterate through all devices in the iterator.
                            // This is REQUIRED to arm the notification for future events.
  loop {
    if count >= max_iterations {
      error!(
        "Device iteration exceeded safety limit of {} iterations!",
        max_iterations
      );
      break;
    }

    let device = unsafe { IOIteratorNext(iterator) };
    if device.0 == 0 {
      // No more devices in iterator.
      debug!("No more devices in iterator (after {} iterations)", count);
      break;
    }

    count += 1;
    info!("Device #{} connected: {:?}", count, device);

    // Extract device properties and set them as state variables
    unsafe {
      extract_and_set_device_properties(device);
    }

    // Emit the Add event to the Rust callback.
    let event = Box::new(DeviceConnectionEvent::Add);
    match refcon_callback(refcon_value, event) {
      Ok(_kernel_port) => {
        debug!("Successfully invoked device callback for device #{}", count);
      }
      Err(e) => {
        error!(
          "Failed to invoke device callback for device #{}: {:?}",
          count, e
        );
      }
    }

    // Release the device reference as required by IOKit.
    unsafe {
      let release_result = IOObjectRelease(device);
      if release_result != 0 {
        error!(
          "IOObjectRelease failed for device {:?} with code: {}",
          device, release_result
        );
      }
    }
  }

  info!(
    "Device connection callback complete. Processed {} devices.",
    count
  );
}

// Drains an iterator without emitting events.
// Used to arm the notification after initial registration.
fn drain_iterator(iterator: io_iterator_t) {
  if iterator.0 == 0 {
    warn!("drain_iterator called with NULL iterator!");
    return;
  }

  debug!("Draining iterator: {:?}", iterator);
  let mut count = 0;
  let max_iterations = 100; // Safety limit

  loop {
    if count >= max_iterations {
      error!(
        "Drain iterator exceeded safety limit of {} iterations!",
        max_iterations
      );
      break;
    }

    let device = unsafe { IOIteratorNext(iterator) };
    if device.0 == 0 {
      break;
    }
    count += 1;
    trace!("Draining existing device #{}: {:?}", count, device);
    unsafe {
      let release_result = IOObjectRelease(device);
      if release_result != 0 {
        error!(
          "IOObjectRelease failed during drain for device {:?} with code: {}",
          device, release_result
        );
      }
    }
  }
  info!(
    "Iterator drained, notification armed. Drained {} existing devices.",
    count
  );
}

pub fn device_connection_listen_start(
  device_class: &str,
  notification_type: &str,
  callback: impl FnMut(DeviceConnectionEvent) -> () + Send + Sync + 'static,
) -> Result<Box<dyn FnOnce() -> Result<(), AppError>>, AppError> {
  // Common device classes:
  // - IOUSBHostDevice: Modern USB devices (macOS 10.11+)
  // - IOUSBDevice: Legacy USB devices (deprecated)
  // - IOMedia: Disk/storage devices (RAM disks, etc.)
  let device_class_cstring = std::ffi::CString::new(device_class).unwrap();

  // Notification types:
  // - FirstPublish: Fires when device is registered (early)
  // - FirstMatch: Fires when drivers are started (late, default)
  let notification_cstring: &[u8] = match notification_type {
    "FirstPublish" => kIOFirstPublishNotification,
    "FirstMatch" => kIOFirstMatchNotification,
    _ => {
      warn!(
        "Unknown notification type '{}', using FirstMatch",
        notification_type
      );
      kIOFirstMatchNotification
    }
  };

  info!(
    "Monitoring device class: {} with notification type: {}",
    device_class, notification_type
  );

  let matching =
    unsafe { IOServiceMatching(device_class_cstring.as_ptr() as *mut i8) };

  if matching.is_null() {
    error!(
      "IOServiceMatching returned NULL for device class '{}'",
      device_class
    );
    return Err(AppError::ListenerRegistrationFailed);
  }

  unsafe {
    debug!(
      "IOServiceMatching created matching dictionary: {:?}",
      *matching
    )
  };

  let matching_for_listen = matching.clone();

  // Use Arc<Mutex<>> to share the iterator value between closure and outer scope
  let initial_iterator = Arc::new(Mutex::new(io_object_t(0)));
  let initial_iterator_clone = initial_iterator.clone();

  let stop_callback = listen_start(
    Box::new(
      move |notifier: &mut io_object_t,
            port_ref: &mut IONotificationPortRef,
            refcon: &mut usize| unsafe {
        let result = IOServiceAddMatchingNotification(
          *port_ref,
          notification_cstring.as_ptr() as *mut i8,
          matching_for_listen,
          Option::Some(device_connection_callback),
          // This is tossed back to the callback, so we know which instance of
          // the callback was invoked.
          *refcon as *mut c_void,
          // This is an io_object_t but an io_iterator_t is expected.  Why
          // does it work?  Probably because they are backed by the same
          // underlying type.
          notifier,
        );
        debug!(
          "IOServiceAddMatchingNotification returned: {} (0 = success)",
          result
        );
        if result != 0 {
          error!(
            "IOServiceAddMatchingNotification failed with code: {}",
            result
          );
        }
        // Save the iterator so we can drain it after listen_start completes.
        *initial_iterator_clone.lock().unwrap() = *notifier;
        debug!("Initial iterator saved: {:?}", *notifier);
        ListenResult::kern_return_t(result)
      },
    ),
    callback,
  )?;

  // CRITICAL: Drain the initial iterator to arm the notification.
  // When IOServiceAddMatchingNotification is first called, the iterator
  // contains all currently connected devices. We must drain it or future
  // notifications will not fire. See Apple IOKit documentation.
  let iterator_to_drain = *initial_iterator.lock().unwrap();
  debug!("About to drain iterator: {:?}", iterator_to_drain);
  drain_iterator(iterator_to_drain as io_iterator_t);

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
