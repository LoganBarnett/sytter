mod macos_bindings;
// mod macos_bindings_emitted;

extern crate num;
#[macro_use]
extern crate num_derive;

pub use crate::macos_bindings::*;
// pub use crate::macos_bindings::{
//   AppleIoMessage,
//   AppleIoReturn,
//   CFNumberCreate,
//   IOAllowPowerChange,
//   kCFNumberSInt32Type,
//   kIOMessageCanSystemSleep,
//   kIOMessageSystemHasPoweredOn,
//   kIOMessageSystemWillNotSleep,
//   kIOMessageSystemWillPowerOn,
//   kIOMessageSystemWillSleep,
// };
// pub use crate::macos_bindings_emitted::*;
