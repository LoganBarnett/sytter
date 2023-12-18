#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

// It looks like there is icrate for doing objective C binding generation that
// might be really good.  I haven't looked into it much, and it would be really
// great if it were also part of bindgen (the author admits as much).  This is
// from the same author as objc2, so I could expect a reasonable amount of
// integration there.  See for the completion:
// https://github.com/madsmtm/objc2/pull/308
// Until then, I'm still using this weirdly merged version of things (that
// somehow works).  But I get these warnings:
// warning: unexpected `cfg` condition value: `cargo-clippy`
//      --> lib/macos-bindings/src/macos_bindings_emitted.rs:65567:19
//       |
// 65567 |     Self(unsafe { msg_send!(class!(aProtocol), alloc) })
//       |                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//       |
//       = note: no expected values for `feature`
//       = help: consider adding `cargo-clippy` as a feature in `Cargo.toml`
//       = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
//       = note: this warning originates in the macro `sel_impl` which comes from the expansion of the macro `msg_send` (in Nightly builds, run with -Z macro-backtrace for more info)
// Which, based on other hardships I've had with upgrading things, is perhaps a
// sign that something changed in Rust proper, and it's caused a bunch of
// downstream warnings that folks have adjusted in their current versions of the
// library, but objc isn't current so it'll never get fixed.  Instead of
// reapproaching things, I've chosen to ignore the warning thusly:
#![allow(unexpected_cfgs)]
// As a side note to the clippy error above, it would be really nice if Cargo
// would tell me what warning is being flagged so I can easily add allow
// statements like above.  The _only_ place this seems to be documented is in
// Rust itself via `rustc -W help`.
#![cfg(target_os = "macos")]
include!("macos_bindings_emitted.rs");
// http://fxr.watson.org/fxr/source/iokit/IOKit/IOReturn.h?v=xnu-1699.24.8#L48
pub const fn err_system(x: u32) -> u32 {
    (x & 0x3f) << 26
}
pub const fn err_sub(x: u32) -> u32 {
    (x & 0xfff) << 14
}
pub const sys_iokit: u32 = err_system(0x38);
pub const sys_iokit_common: u32 = err_sub(0x0);

pub const fn iokit_common_msg(message: u32) -> u32 {
    sys_iokit | sys_iokit_common | message
}

pub const fn iokit_common_err(sub: u32) -> u32 {
    sys_iokit | sub
}
// Taken from:
// https://opensource.apple.com/source/xnu/xnu-4570.71.2/iokit/IOKit/IOMessage.h
// The can be broken down thusly:
// https://gist.github.com/MLKrisJohnson/eb5e1cb623694372676c938be82c9bb4
// Bindings don't seem to get generated here.  Probably because they are
// declared as iokit_common_msg(message) which itself is a macro.  It's done
// indescriminately, so it's more than what we need.
pub const kIOMessageServiceIsTerminated: u32 = iokit_common_msg(0x010);
pub const kIOMessageServiceIsSuspended: u32 = iokit_common_msg(0x020);
pub const kIOMessageServiceIsResumed: u32 = iokit_common_msg(0x030);
pub const kIOMessageServiceIsRequestingClose: u32 = iokit_common_msg(0x100);
pub const kIOMessageServiceIsAttemptingOpen: u32 = iokit_common_msg(0x101);
pub const kIOMessageServiceWasClosed: u32 = iokit_common_msg(0x110);
pub const kIOMessageServiceBusyStateChange: u32 = iokit_common_msg(0x120);
pub const kIOMessageCanDevicePowerOff: u32 = iokit_common_msg(0x200);
pub const kIOMessageDeviceWillPowerOff: u32 = iokit_common_msg(0x210);
pub const kIOMessageDeviceWillNotPowerOff: u32 = iokit_common_msg(0x220);
pub const kIOMessageDeviceHasPoweredOn: u32 = iokit_common_msg(0x230);
pub const kIOMessageCanSystemPowerOff: u32 = iokit_common_msg(0x240);
pub const kIOMessageSystemWillPowerOff: u32 = iokit_common_msg(0x250);
pub const kIOMessageSystemWillNotPowerOff: u32 = iokit_common_msg(0x260);
pub const kIOMessageCanSystemSleep: u32 = iokit_common_msg(0x270);
pub const kIOMessageSystemWillSleep: u32 = iokit_common_msg(0x280);
pub const kIOMessageSystemWillNotSleep: u32 = iokit_common_msg(0x290);
pub const kIOMessageSystemHasPoweredOn: u32 = iokit_common_msg(0x300);
pub const kIOMessageSystemWillRestart: u32 = iokit_common_msg(0x310);
pub const kIOMessageSystemWillPowerOn: u32 = iokit_common_msg(0x320);

// Adapted from
// http://fxr.watson.org/fxr/source/iokit/IOKit/IOReturn.h?v=xnu-1699.24.8#L74
pub const kIOReturnError: u32 = iokit_common_err(0x2bc); // general error
pub const kIOReturnNoMemory: u32 = iokit_common_err(0x2bd); // can't allocate memory
pub const kIOReturnNoResources: u32 = iokit_common_err(0x2be); // resource shortage
pub const kIOReturnIPCError: u32 = iokit_common_err(0x2bf); // error during IPC
pub const kIOReturnNoDevice: u32 = iokit_common_err(0x2c0); // no such device
pub const kIOReturnNotPrivileged: u32 = iokit_common_err(0x2c1); // privilege violation
pub const kIOReturnBadArgument: u32 = iokit_common_err(0x2c2); // invalid argument
pub const kIOReturnLockedRead: u32 = iokit_common_err(0x2c3); // device read locked
pub const kIOReturnLockedWrite: u32 = iokit_common_err(0x2c4); // device write locked
pub const kIOReturnExclusiveAccess: u32 = iokit_common_err(0x2c5); // exclusive access and
                                                                   //   device already open
pub const kIOReturnBadMessageID: u32 = iokit_common_err(0x2c6); // sent/received messages
                                                                //   had different msg_id
pub const kIOReturnUnsupported: u32 = iokit_common_err(0x2c7); // unsupported function
pub const kIOReturnVMError: u32 = iokit_common_err(0x2c8); // misc. VM failure
pub const kIOReturnInternalError: u32 = iokit_common_err(0x2c9); // internal error
pub const kIOReturnIOError: u32 = iokit_common_err(0x2ca); // General I/O error
                                                           //#define kIOReturn???Error      iokit_common_err(0x2cb) // ???
pub const kIOReturnCannotLock: u32 = iokit_common_err(0x2cc); // can't acquire lock
pub const kIOReturnNotOpen: u32 = iokit_common_err(0x2cd); // device not open
pub const kIOReturnNotReadable: u32 = iokit_common_err(0x2ce); // read not supported
pub const kIOReturnNotWritable: u32 = iokit_common_err(0x2cf); // write not supported
pub const kIOReturnNotAligned: u32 = iokit_common_err(0x2d0); // alignment error
pub const kIOReturnBadMedia: u32 = iokit_common_err(0x2d1); // Media Error
pub const kIOReturnStillOpen: u32 = iokit_common_err(0x2d2); // device(s) still open
pub const kIOReturnRLDError: u32 = iokit_common_err(0x2d3); // rld failure
pub const kIOReturnDMAError: u32 = iokit_common_err(0x2d4); // DMA failure
pub const kIOReturnBusy: u32 = iokit_common_err(0x2d5); // Device Busy
pub const kIOReturnTimeout: u32 = iokit_common_err(0x2d6); // I/O Timeout
pub const kIOReturnOffline: u32 = iokit_common_err(0x2d7); // device offline
pub const kIOReturnNotReady: u32 = iokit_common_err(0x2d8); // not ready
pub const kIOReturnNotAttached: u32 = iokit_common_err(0x2d9); // device not attached
pub const kIOReturnNoChannels: u32 = iokit_common_err(0x2da); // no DMA channels left
pub const kIOReturnNoSpace: u32 = iokit_common_err(0x2db); // no space for data
                                                           // pub const kIOReturn???Error      iokit_common_err(0x2dc) // ???
pub const kIOReturnPortExists: u32 = iokit_common_err(0x2dd); // port already exists
pub const kIOReturnCannotWire: u32 = iokit_common_err(0x2de); // can't wire down
                                                              //   physical memory
pub const kIOReturnNoInterrupt: u32 = iokit_common_err(0x2df); // no interrupt attached
pub const kIOReturnNoFrames: u32 = iokit_common_err(0x2e0); // no DMA frames enqueued
pub const kIOReturnMessageTooLarge: u32 = iokit_common_err(0x2e1); // oversized msg received
                                                                   //   on interrupt port
pub const kIOReturnNotPermitted: u32 = iokit_common_err(0x2e2); // not permitted
pub const kIOReturnNoPower: u32 = iokit_common_err(0x2e3); // no power to device
pub const kIOReturnNoMedia: u32 = iokit_common_err(0x2e4); // media not present
pub const kIOReturnUnformattedMedia: u32 = iokit_common_err(0x2e5); // media not formatted
pub const kIOReturnUnsupportedMode: u32 = iokit_common_err(0x2e6); // no such mode
pub const kIOReturnUnderrun: u32 = iokit_common_err(0x2e7); // data underrun
pub const kIOReturnOverrun: u32 = iokit_common_err(0x2e8); // data overrun
pub const kIOReturnDeviceError: u32 = iokit_common_err(0x2e9); // the device is not working properly!
pub const kIOReturnNoCompletion: u32 = iokit_common_err(0x2ea); // a completion routine is required
pub const kIOReturnAborted: u32 = iokit_common_err(0x2eb); // operation aborted
pub const kIOReturnNoBandwidth: u32 = iokit_common_err(0x2ec); // bus bandwidth would be exceeded
pub const kIOReturnNotResponding: u32 = iokit_common_err(0x2ed); // device not responding
pub const kIOReturnIsoTooOld: u32 = iokit_common_err(0x2ee); // isochronous I/O request for distant past!
pub const kIOReturnIsoTooNew: u32 = iokit_common_err(0x2ef); // isochronous I/O request for distant future
pub const kIOReturnNotFound: u32 = iokit_common_err(0x2f0); // data was not found
pub const kIOReturnInvalid: u32 = iokit_common_err(0x1); // should never be seen

// https://developer.apple.com/documentation/corefoundation/cfnumbertype/kcfnumbersint32type
pub const kCFNumberSInt32Type: i64 = 0x3;

#[derive(Debug, FromPrimitive)]
// Needed due to
// https://github.com/rust-lang/rust/issues/21493#issuecomment-71304090
// but the error still isn't helpful.
#[repr(u32)]
pub enum AppleIoReturn {
    kIOReturnSuccess = kIOReturnSuccess,
    kIOReturnError = kIOReturnError,
    kIOReturnNoMemory = kIOReturnNoMemory,
    kIOReturnNoResources = kIOReturnNoResources,
    kIOReturnIPCError = kIOReturnIPCError,
    kIOReturnNoDevice = kIOReturnNoDevice,
    kIOReturnNotPrivileged = kIOReturnNotPrivileged,
    kIOReturnBadArgument = kIOReturnBadArgument,
    kIOReturnLockedRead = kIOReturnLockedRead,
    kIOReturnLockedWrite = kIOReturnLockedWrite,
    kIOReturnExclusiveAccess = kIOReturnExclusiveAccess,
    kIOReturnBadMessageID = kIOReturnBadMessageID,
    kIOReturnUnsupported = kIOReturnUnsupported,
    kIOReturnVMError = kIOReturnVMError,
    kIOReturnInternalError = kIOReturnInternalError,
    kIOReturnIOError = kIOReturnIOError,
    kIOReturnCannotLock = kIOReturnCannotLock,
    kIOReturnNotOpen = kIOReturnNotOpen,
    kIOReturnNotReadable = kIOReturnNotReadable,
    kIOReturnNotWritable = kIOReturnNotWritable,
    kIOReturnNotAligned = kIOReturnNotAligned,
    kIOReturnBadMedia = kIOReturnBadMedia,
    kIOReturnStillOpen = kIOReturnStillOpen,
    kIOReturnRLDError = kIOReturnRLDError,
    kIOReturnDMAError = kIOReturnDMAError,
    kIOReturnBusy = kIOReturnBusy,
    kIOReturnTimeout = kIOReturnTimeout,
    kIOReturnOffline = kIOReturnOffline,
    kIOReturnNotReady = kIOReturnNotReady,
    kIOReturnNotAttached = kIOReturnNotAttached,
    kIOReturnNoChannels = kIOReturnNoChannels,
    kIOReturnNoSpace = kIOReturnNoSpace,
    kIOReturnPortExists = kIOReturnPortExists,
    kIOReturnCannotWire = kIOReturnCannotWire,
    kIOReturnNoInterrupt = kIOReturnNoInterrupt,
    kIOReturnNoFrames = kIOReturnNoFrames,
    kIOReturnMessageTooLarge = kIOReturnMessageTooLarge,
    kIOReturnNotPermitted = kIOReturnNotPermitted,
    kIOReturnNoPower = kIOReturnNoPower,
    kIOReturnNoMedia = kIOReturnNoMedia,
    kIOReturnUnformattedMedia = kIOReturnUnformattedMedia,
    kIOReturnUnsupportedMode = kIOReturnUnsupportedMode,
    kIOReturnUnderrun = kIOReturnUnderrun,
    kIOReturnOverrun = kIOReturnOverrun,
    kIOReturnDeviceError = kIOReturnDeviceError,
    kIOReturnNoCompletion = kIOReturnNoCompletion,
    kIOReturnAborted = kIOReturnAborted,
    kIOReturnNoBandwidth = kIOReturnNoBandwidth,
    kIOReturnNotResponding = kIOReturnNotResponding,
    kIOReturnIsoTooOld = kIOReturnIsoTooOld,
    kIOReturnIsoTooNew = kIOReturnIsoTooNew,
    kIOReturnNotFound = kIOReturnNotFound,
    kIOReturnInvalid = kIOReturnInvalid,
}

#[derive(Debug, FromPrimitive)]
// Needed due to
// https://github.com/rust-lang/rust/issues/21493#issuecomment-71304090
// but the error still isn't helpful.
#[repr(u32)]
pub enum AppleIoMessage {
    kIOMessageServiceIsTerminated = kIOMessageServiceIsTerminated,
    kIOMessageServiceIsSuspended = kIOMessageServiceIsSuspended,
    kIOMessageServiceIsResumed = kIOMessageServiceIsResumed,
    kIOMessageServiceIsRequestingClose = kIOMessageServiceIsRequestingClose,
    kIOMessageServiceIsAttemptingOpen = kIOMessageServiceIsAttemptingOpen,
    kIOMessageServiceWasClosed = kIOMessageServiceWasClosed,
    kIOMessageServiceBusyStateChange = kIOMessageServiceBusyStateChange,
    kIOMessageCanDevicePowerOff = kIOMessageCanDevicePowerOff,
    kIOMessageDeviceWillPowerOff = kIOMessageDeviceWillPowerOff,
    kIOMessageDeviceWillNotPowerOff = kIOMessageDeviceWillNotPowerOff,
    kIOMessageDeviceHasPoweredOn = kIOMessageDeviceHasPoweredOn,
    kIOMessageCanSystemPowerOff = kIOMessageCanSystemPowerOff,
    kIOMessageSystemWillPowerOff = kIOMessageSystemWillPowerOff,
    kIOMessageSystemWillNotPowerOff = kIOMessageSystemWillNotPowerOff,
    kIOMessageCanSystemSleep = kIOMessageCanSystemSleep,
    kIOMessageSystemWillSleep = kIOMessageSystemWillSleep,
    kIOMessageSystemWillNotSleep = kIOMessageSystemWillNotSleep,
    kIOMessageSystemHasPoweredOn = kIOMessageSystemHasPoweredOn,
    kIOMessageSystemWillRestart = kIOMessageSystemWillRestart,
    kIOMessageSystemWillPowerOn = kIOMessageSystemWillPowerOn,
}

extern "C" {
    // We need this function but Rust won't let us do a raw cast to accept a
    // *c_void, which the function actually expects.  Without this we run into
    // problems where Rust is using some kind of wrapped value.  We can't pass
    // the original value to the function and so we never actually do the allow.
    // See build.rs for the exclude from the bindgen generation.
    pub fn IOAllowPowerChange(
        kernelPort: io_connect_t,
        notificationID: *mut ::std::os::raw::c_void,
    ) -> IOReturn;
}

// Unsure why this isn't getting picked up anymore.  I bumped the bindgen
// package and didn't look to see if anything changed of importance on
// invocation.  For now just copy the declaration here until I can figure it
// out.
extern "C" {
  pub fn CFNumberCreate(
    allocator: CFAllocatorRef,
    theType: CFNumberType,
    valuePtr: *const ::std::os::raw::c_void,
  ) -> CFNumberRef;
}

pub type CFNumberType = CFIndex;
pub type CFNumberRef = *const __CFNumber;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __CFNumber {
  _unused: [u8; 0],
}
