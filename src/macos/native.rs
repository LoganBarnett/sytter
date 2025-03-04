/*******************************************************************************
 * This file contains any shared functionality between the various macOS native
 * operations that is performed by Sytter.
 ******************************************************************************/
use crate::{
  error::AppError,
};
use sytter_macos_bindings::{
  __CFDictionary,
  // kern_return_t,
  kCFAllocatorDefault,
  kCFNumberSInt32Type,
  CFDictionaryGetCount,
  CFDictionaryGetKeysAndValues,
  CFDictionarySetValue,
  CFNumberCreate,
  CFRelease,
  // IOMainPort,
  // This is the old identifier as seen in many examples, but it's been
  // changed to IOMainPort in recent editions of macOS.  However this doesn't
  // seem to hold for arm64/aarch64 systems.
  IOMasterPort,
  IONotificationPortCreate,
  IONotificationPortRef,
  MACH_PORT_NULL,
};
use log::*;
use std::ffi::{
  c_void,
  // CString,
  // CStr,
};

#[allow(unused)]
trait CustomCfDictionary {
  fn len(&self) -> usize;
  fn get_keys_and_values(&self) -> (Vec<*const c_void>, Vec<*const c_void>);
}

impl CustomCfDictionary for __CFDictionary {

  fn len(&self) -> usize {
    unsafe { CFDictionaryGetCount(self).try_into().unwrap() }
  }

  // Shameless lift from:
  // https://github.com/servo/core-foundation-rs/blob/3570256eccb0cc4945f7d5fee08d1d14df865bf6/core-foundation/src/dictionary.rs#L272C1-L284C6
  fn get_keys_and_values(&self) -> (Vec<*const c_void>, Vec<*const c_void>) {
    let length = self.len();
    let mut keys = Vec::with_capacity(length);
    let mut values = Vec::with_capacity(length);
    unsafe {
      CFDictionaryGetKeysAndValues(
        self,
        keys.as_mut_ptr(),
        values.as_mut_ptr(),
      );
      keys.set_len(length);
      values.set_len(length);
    }
    (keys, values)
  }

  // fn fmt_2(&self, f: std::fmt::Formatter<'_>) -> std::fmt::Result {
  //   // // let keys_ptr = std::ptr::null_mut();
  //   // // let values_ptr = std::ptr::null_mut();
  //   // let count = CFDictionaryGetCount(self);
  //   // // MaybeUninit is what is recommended here:
  //   // // https://users.rust-lang.org/t/ffi-how-to-pass-a-array-with-structs-to-a-c-func-that-fills-the-array-out-pointer-and-then-how-to-access-the-items-after-in-my-rust-code/83798/2
  //   // let mut keys_ptr = MaybeUninit::uninit();
  //   // let mut values_ptr = MaybeUninit::uninit();
  //   // CFDictionaryGetKeysAndValues(self, keys_ptr, values_ptr);
  //   // let keys = keys_ptr as [usize; count];
  //   // let values = values_ptr as [*mut c_void; count];
  //   let (keys, values) = self.get_keys_and_values();
  //   let debug_thingy = f.debug_struct("CFDictionary");
  //   for (k, v) in keys
  //     .into_iter()
  //     .zip(values.into_iter())
  //   {
  //     debug_thingy.field(k, &v);
  //   }
  //   debug_thingy.finish()
  // }

}

// impl Into<CString> for [u8] {
// }

// Kept because some documentation indiciated this is what we should do, but
// examples don't use it, and it segfaults when called.
#[allow(dead_code)]
pub fn mach_port() -> Result<u32, AppError> {
    trace!("In mach_port.");
    let mach_port = std::ptr::null_mut();
    // It is not documented elsewhere.  I should be able just look for 0.
    // TODO: Use IOMasterPort for macos <= 12.
    trace!("IOMainPort({:?}, {:?})", MACH_PORT_NULL, mach_port);
    let io_return = unsafe { IOMasterPort(MACH_PORT_NULL, mach_port) };
    trace!("io_return is: {:?}", io_return);
    if io_return == 0 {
        return Err(AppError::MachPortRegistrationFailed());
    }
    let written_mach_port: u32 = unsafe { *mach_port };
    trace!("mach_port is {:?}", written_mach_port);
    Ok(written_mach_port)
}

pub fn port_ref_create() -> Result<IONotificationPortRef, AppError> {
  // let io_main_port = mach_port()?;
  let port_ref: IONotificationPortRef = unsafe {
    IONotificationPortCreate(0)
    // IONotificationPortCreate(io_main_port.try_into().unwrap())
  };
  trace!("port_ref: {:?}", port_ref);
  Ok(port_ref)
}

// TODO: Figure out types.
#[allow(dead_code)]
pub fn dict_set_i32(
  dict: *mut __CFDictionary,
  key: *mut c_void,
  value: i32,
) -> () {
  let number_ref = unsafe { CFNumberCreate(
    kCFAllocatorDefault,
    kCFNumberSInt32Type,
    value as *mut c_void,
  ) };
  unsafe { CFDictionarySetValue(
    dict,
    key,
    number_ref as *mut c_void,
  )};
  unsafe { CFRelease(number_ref as *mut c_void) };
}
