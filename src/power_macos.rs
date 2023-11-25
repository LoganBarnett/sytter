#![cfg(target_os = "macos")]

use crate::num::FromPrimitive;
use crate::{
    macos_bindings::{
        AppleIoMessage,
        AppleIoReturn,
        CFRunLoopAddSource,
        CFRunLoopGetCurrent,
        CFRunLoopRun,
        CFRunLoopSourceRef,
        IOAllowPowerChange,
        IODeregisterForSystemPower,
        IOMainPort,
        // IOMasterPort,
        IONotificationPortCreate,
        IONotificationPortDestroy,
        IONotificationPortGetRunLoopSource,
        IONotificationPortRef,
        IORegisterForSystemPower,
        MACH_PORT_NULL,
        UInt32,
        io_connect_t,
        io_object_t,
        io_service_t,
        kCFRunLoopCommonModes,
        kIOMessageCanSystemSleep,
        kIOMessageSystemHasPoweredOn,
        kIOMessageSystemWillNotSleep,
        kIOMessageSystemWillPowerOn,
        kIOMessageSystemWillSleep,
        kIOReturnSuccess,
    },
    error::AppError,
};
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
use std::thread;
/**
 * For Darwin (macOS), see
 * https://developer.apple.com/documentation/iokit/1557114-ioregisterforsystempower
 * for some relevant documentation.
 * This also appears to be an information trove:
 * https://docs.darlinghq.org/internals/macos-specifics/mach-ports.html
 */
// https://developer.apple.com/documentation/iokit/ioserviceinterestcallback
// io_connect_t IORegisterForSystemPower(void *refcon, IONotificationPortRef *thePortRef, IOServiceInterestCallback callback, io_object_t *notifier);
use std::{collections::HashMap, ffi::c_void};
// Gives us macros such as debug! and error! See logging.rs for setup.
use log::*;

type KernelPortCallback = dyn FnMut() -> io_connect_t + Send;

lazy_static! {
    static ref CALLBACKS: Arc<Mutex<HashMap<usize, Box<KernelPortCallback>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

#[cfg(target_os = "macos")]
extern "C" fn service_callback(
    refcon: *mut c_void,
    _service: io_service_t,
    message_type: UInt32,
    message_argument: *mut c_void,
) -> () {
    debug!(
        "Got macOS power event! {:?}",
        AppleIoMessage::from_u32(message_type as u32).unwrap(),
    );
    trace!("refcon: {:?}", refcon);
    // These two messages require acknowledgement or they will stall the
    // sleeping process.
    if message_type == kIOMessageCanSystemSleep
        || message_type == kIOMessageSystemWillSleep
    {
        if message_type == kIOMessageCanSystemSleep {
            debug!("Got message kIOMessageCanSystemSleep.");
        } else {
            debug!("Got message kIOMessageWillSystemSleep.");
        }
        trace!("Getting {:?} from callback container.", refcon);
        let num = refcon as usize;
        let mut callbacks = CALLBACKS.lock().unwrap();
        let closure: &mut Box<KernelPortCallback> =
            callbacks.get_mut(&num).unwrap();
        let kernel_port = closure();
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
This will keep the machine from sleeping for 30+ seconds. \
Using kernel_port {:?} and notifiy_id {:?}",
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
    }
    // These should be no action - someone vetoed sleep.
    else if message_type == kIOMessageSystemWillNotSleep {
        debug!("Got message kIOMessageSystemWillNotSleep.");
    }
}

// Kept because some documentation indiciated this is what we should do, but
// examples don't use it, and it segfaults when called.
#[allow(dead_code)]
fn mach_port() -> Result<u32, AppError> {
    trace!("In mach_port.");
    let mach_port = std::ptr::null_mut();
    // It is not documented elsewhere.  I should be able just look for 0.
    // TODO: Use IOMasterPort for macos <= 12.
    trace!("IOMainPort({:?}, {:?})", MACH_PORT_NULL, mach_port);
    let io_return = unsafe { IOMainPort(MACH_PORT_NULL, mach_port) };
    trace!("io_return is: {:?}", io_return);
    if io_return == 0 {
        return Err(AppError::PowerHookRegistrationFailed);
    }
    let written_mach_port: u32 = unsafe { *mach_port };
    trace!("mach_port is {:?}", written_mach_port);
    Ok(written_mach_port)
}

fn port_ref_create() -> Result<IONotificationPortRef, AppError> {
    let port_ref: IONotificationPortRef = unsafe {
        IONotificationPortCreate(0)
    };
    trace!("port_ref: {:?}", port_ref);
    Ok(port_ref)
}

#[cfg(target_os = "macos")]
pub fn sleep_listen_start(
    mut callback: impl FnMut() -> () + Send + 'static,
) -> Result<Box<dyn FnOnce() -> Result<(), AppError>>, AppError> {
    let mut port_ref = port_ref_create()?;
    let mut notifier: io_object_t = io_object_t(0u32);
    // This value is required later to be used for IOAllowPowerChange - a
    // function that must be invoked to acknowledge work is done with our
    // service before sleep can properly begin.  It doesn't matter if our
    // service doesn't do anything with the event - the event must be
    // acknowledged regardless.
    let mut kernel_port: io_connect_t =
        io_connect_t(io_object_t(MACH_PORT_NULL));
    trace!("kernel_port: {:?}", kernel_port);
    // port_ref gets mutated here, so copy it so we can use it for our callback
    // identification.
    let port_ref_to_refcon = port_ref.clone();
    kernel_port = unsafe {
        IORegisterForSystemPower(
            // This is tossed back to the callback, so we know which instance of
            // the callback was invoked.
            *port_ref as *mut c_void,
            &mut port_ref,
            Option::Some(service_callback),
            &mut notifier,
        )
    };
    trace!("kernel_port mutated to: {:?}", kernel_port);
    trace!("port_ref mutated to: {:?}", port_ref);
    let stoppable_callback: Box<KernelPortCallback> = Box::new(move || {
        trace!("In closure!");
        callback();
        kernel_port
    });
    trace!("Storing {:?} in callbacks container.", port_ref_to_refcon.0);
    {
        let mut callbacks = CALLBACKS.lock().unwrap();
        callbacks.insert(port_ref_to_refcon.0 as usize, stoppable_callback);
    }
    let loop_source = unsafe {
        IONotificationPortGetRunLoopSource(port_ref)
    } as usize; // usize the pointer so we can toss it over the thread "safely".
    trace!("loop_source {:?}", loop_source);
    // io_object_t is a self-referencing tuple, so we have to reach in twice to
    // get the underlying value.
    if kernel_port.0 .0 == MACH_PORT_NULL {
        Err(AppError::PowerHookRegistrationFailed)
    } else {
        unsafe {
            let _scheduler = thread::spawn(move || {
                trace!(
                    "In thread for CFRunLoop with loop_source {:?}.",
                    loop_source,
                );
                CFRunLoopAddSource(
                    CFRunLoopGetCurrent(),
                    loop_source as CFRunLoopSourceRef,
                    kCFRunLoopCommonModes,
                );
                trace!("CFRunLoop registered.  Waiting for callbacks.");
                // This blocks, and is necessary for callbacks to be invoked.
                // Without this, there will be no warning and log _anywhere_
                // that the message was not posted or that there is otherwise a
                // problem.  This includes both on this app's side as well as
                // the system logs (seen via Console.app).
                CFRunLoopRun();
                trace!("Unreachable? Done with loop run.");
            });
        }
        Ok(Box::new(move || sleep_listen_stop(notifier, port_ref)))
    }
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
