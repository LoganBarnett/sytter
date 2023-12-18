#![cfg(target_os = "macos")]
/**
 * Tools for handling events from macOS via registering listeners (macOS terms
 * as "notifications").
 */
use crate::{
  error::AppError,
  macos::native::port_ref_create,
};
use sytter_macos_bindings::{
  CFRunLoopAddSource,
  CFRunLoopGetCurrent,
  CFRunLoopRun,
  CFRunLoopSourceRef,
  IONotificationPortGetRunLoopSource,
  IONotificationPortRef,
  MACH_PORT_NULL,
  UInt32,
  io_connect_t,
  io_object_t,
  io_service_t,
  kern_return_t,
  kCFRunLoopCommonModes,
};
use lazy_static::lazy_static;
use log::*;
use std::any::{Any, type_name};
use std::fmt::Debug;
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

pub trait CallbackData: Clone + Debug {
  // fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum ListenResult {
  io_connect_t(io_connect_t),
  kern_return_t(kern_return_t),
}

impl ListenResult {

  pub fn success(&self) -> bool {
    match self {
      // io_object_t is a self-referencing tuple, so we have to reach in twice
      // to get the underlying value.
      ListenResult::io_connect_t(io) => io.0.0 == MACH_PORT_NULL,
      ListenResult::kern_return_t(k) => *k == 0,
    }
  }

}

pub type _ServiceCallback = dyn FnOnce(
  *mut c_void,
  io_service_t,
  UInt32,
  *mut c_void,
) -> ();

pub type ListenStartCallback = dyn FnMut(
  &mut io_object_t,
  &mut IONotificationPortRef,
  &mut usize,
) -> ListenResult;

pub type _ListenStopCallback = dyn FnMut(
  &mut io_object_t,
  &mut IONotificationPortRef,
) -> () + Send;

// pub type ListenTranslateCallback<A: CallbackData> = dyn FnMut(
//   A
// ) -> () + Send + Sync;

pub type _ListenCallback = dyn FnMut(
  Box<dyn Any>,
) -> () + Send + Sync;

pub type KernelPortCallback = dyn FnMut(
  Box<dyn Any>,
) -> io_connect_t + Send + Sync;

lazy_static! {
  pub static ref CALLBACKS: Arc<Mutex<HashMap<usize, Box<KernelPortCallback>>>> =
    Arc::new(Mutex::new(HashMap::new()));
}

pub fn listen_start<A: CallbackData + 'static>(
  mut listen_fn: Box<ListenStartCallback>,
  mut callback: impl FnMut(A) -> () + Send + Sync + 'static,
) -> Result<Box<dyn FnOnce() -> (io_object_t, IONotificationPortRef)>, AppError> {
  // This value is required later to be used _by_ the callback, for example
  // IOAllowPowerChange - a function that must be invoked to acknowledge
  // work is done with our service before sleep can properly begin.  It
  // doesn't matter if our service doesn't do anything with the event - the
  // event must be acknowledged regardless.
  let mut kernel_port: io_connect_t = io_connect_t(io_object_t(MACH_PORT_NULL));
  trace!("kernel_port: {:?}", kernel_port);
  let mut port_ref = port_ref_create()?;
  let mut notifier: io_object_t = io_object_t(0u32);
  // port_ref gets mutated here, so copy it so we can use it for our
  // callback identification.
  trace!("port_ref before clone: {:x?}", port_ref);
  // let port_ref_to_refcon: usize = port_ref.deref();
  let mut port_ref_to_refcon: usize = rand::random();
  trace!("port_ref after clone:  {:x?}", port_ref_to_refcon);
  let listen_result = listen_fn(
    &mut notifier,
    &mut port_ref,
    &mut port_ref_to_refcon,
  );
  trace!("kernel_port mutated to: {:?}", listen_result);
  match listen_result {
    ListenResult::io_connect_t(kr) => kernel_port = kr,
    _ => (),
  };
  trace!("port_ref mutated to: {:?}", port_ref);
  let stoppable_callback: Box<KernelPortCallback> = Box::new(
    move |data: Box<dyn Any>| {
      trace!("In closure with event {:?}!", data);
      let event: Box<A> = match data
        .downcast_ref::<A>() {
          Some(x) => Box::new(x.clone()),
          None => panic!("{:?} is not a {}!", data, type_name::<A>()),
        };
      callback(*event);
      kernel_port
    }
  );
  trace!("Storing {:x?} in callbacks container.", port_ref_to_refcon);
  {
    let mut callbacks = CALLBACKS
      .lock()
      .inspect(|_| { trace!("Got lock for CALLBACKs."); })
      .map_err(|_e| {
        AppError::EventMutexLockError(
          "Could not lock CALLBACKS.".into(),
        )
      })?;
    callbacks.insert(port_ref_to_refcon, stoppable_callback);
  }
  let loop_source = unsafe {
    IONotificationPortGetRunLoopSource(port_ref)
  }
  // usize the pointer so we can toss it over the thread "safely".
  as usize;
  trace!("loop_source {:x?}", loop_source);
  if false && !listen_result.success() {
    Err(AppError::ListenerRegistrationFailed)
  } else {
    unsafe {
      let _scheduler = thread::spawn(move || {
        trace!("In thread for CFRunLoop with loop_source {:x?}.", loop_source);
        CFRunLoopAddSource(
          CFRunLoopGetCurrent(),
          loop_source as CFRunLoopSourceRef,
          kCFRunLoopCommonModes,
        );
        trace!("CFRunLoop registered.  Waiting for callbacks.");
        // This blocks, and is necessary for callbacks to be invoked.
        // Without this, there will be no warning and log _anywhere_ that
        // the message was not posted or that there is otherwise a problem.
        // This includes both on this app's side as well as the system logs
        // (seen via Console.app).
        CFRunLoopRun();
        trace!("Unreachable? Done with loop run.");
      });
    }
    Ok(Box::new(move || (notifier, port_ref)))
  }
}

pub fn refcon_callback(
  refcon: usize,
  data: Box<dyn Any>,
) -> Result<io_connect_t, AppError> {
  trace!("refcon: {:x?}", refcon);
  trace!("Getting {:x?} from callback container.", refcon);
  let mut callbacks = CALLBACKS
    .lock()
    .inspect(|_| { trace!("Got lock for CALLBACKs."); })
    .map_err(|_e| {
      AppError::EventMutexLockError(
        "Could not lock CALLBACKS.".into(),
      )
    })?;
  // Ugh.  Why, Rust?
  // Even though we might not even use it due to the code path and the trace!
  // macro, we still have to compute a value here or Rust throws a fit over
  // borrows.  A more knowledgeable person could figure this out possibly, but
  // I've had no lock trying to get it working directly in the inspect_err.
  let keys = format!("{:x?}", callbacks.keys().clone());
  let closure: &mut Box<KernelPortCallback> = callbacks
    .get_mut(&refcon)
    .ok_or(AppError::KernelPortCallbackNotFoundError(refcon))
    .inspect_err(|e| {
      info!(
        "CALLBACKS has available: {}\nWe want: {:x?}",
        keys,
        e,
      );
    })
    .inspect(|_| {
      trace!("Found callback for refcon `{:x?}'.", refcon);
    })
    ?;
  Ok(closure(data))
}

// This is unrolled in listen_start.  I couldn't satisfy the type signature.
// pub fn event_propagate<'a, A: CallbackData + 'a + 'static>(
//   mut callback: impl FnMut(A) -> () + Send + Sync + 'static,
//   data: Box<dyn Any>,
// ) -> () {
//   let event: Box<A> = match data
//     .downcast_ref::<A>() {
//       Some(x) => Box::new(x.clone()),
//       None => panic!("{:?} is not a {}!", data, type_name::<A>()),
//     };
//   callback(*event);
// }
