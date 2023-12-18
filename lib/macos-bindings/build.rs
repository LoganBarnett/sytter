
use std::env::{self, VarError};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::string::FromUtf8Error;
use log::{*, Level};
use bindgen::BindgenError;

#[derive(Debug)]
/**
 * A newer version of Rust started producing weird warnings here. See
 * https://github.com/rust-lang/rust/pull/124460 where the fix has been applied
 * but hasn't been consumed yet on this project (2020-07-13).  Until then we
 * ignore dead code, which will, sadly, ignore unused variants as well.
 */
#[allow(dead_code)]
enum BuildError {
  BindingFileWriteError(std::io::Error),
  BindGenGenerateError(BindgenError),
  EncodeError(FromUtf8Error),
  EnvVarError(VarError),
  LoggingInitializationError(log::SetLoggerError),
  ProveBuildRsIsRunningError(),
}

/**
 * Generate the FFI bindings for a given macOS framework and emit them to a
 * src/macos_bindings_emitted_<framework>.rs file.
 */
#[cfg(target_os = "macos")]
fn macos_framework_generate(
  sdk_path: &str,
  frameworks_path: &str,
) -> Result<(), BuildError> {
  // Tell cargo to tell rustc to link the system power management shared
  // library.
  println!("cargo:rustc-link-lib=framework=IOKit");

  println!("cargo::warning='Generating macOS bindings.  This takes a really long time!'");
  let bindings_res = bindgen::Builder::default()
    // time.h as has a variable called timezone that conflicts with some of the
    // objective-c calls from NSCalendar.h in the Foundation framework.  This
    // removes that one variable.
    .blocklist_item("timezone")
    // https://github.com/rust-lang/rust-bindgen/issues/1705
    .blocklist_item("IUIStepper")
    .blocklist_function("dividerImageForLeftSegmentState_rightSegmentState_")
    .blocklist_function(r".*conformsToProtocol.*")
    .blocklist_function(r".*conformsToProtocol_.*")
    // We need this function but Rust won't let us do a raw cast to accept a
    // *c_void, which the function actually expects.  Without this we run into
    // problems where Rust is using some kind of wrapped value.  We can't pass
    // the original value to the function and so we never actually do the allow.
    // See macos_bindings.rs for the manual declaration.
    .blocklist_function(r".*IOAllowPowerChange.*")
    .blocklist_item(r".*conformsToProtocol.*")
    .blocklist_item(r".*conformsToProtocol_.*")
    .new_type_alias_deref(r".*io_object_t.*")
    .new_type_alias_deref(r".*IONotificationPortRef.*")
    .new_type_alias_deref(r".*io_connect_t.*")
    // .blocklist_item("PNSObject")
    // Defaults to this, but this future proofs us.
    .allowlist_recursively(true)
    // .allowlist_file(r"IOKit")
    // The input header we would like to generate bindings for.
    .header("wrapper.h")
    // .header_contents("IOKit.h", "#include <IOKit/pwr_mgt/IOPMLib.h>")
    // .clang_arg("-xobjective-c")
    .clang_arg("-xobjective-c")
    // .clang_arg("-xc++-header")
    .clang_arg(format!("-isysroot{}", sdk_path.trim()))
    // Why do we want blocks?
    .clang_arg("-fblocks")
    // This probably isn't a safe assumption to make.
    .clang_arg("--target=aarch64-apple-macos")
    .clang_arg(format!("-F{}", frameworks_path))
    .objc_extern_crate(false)
    // Tell cargo to invalidate the built crate whenever any of the
    // included header files changed.
    .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
    // Finish the builder and generate the bindings.
    .generate()
    .map_err(BuildError::BindGenGenerateError)
    // Unwrap the Result and panic on failure.
    // .expect("Unable to generate bindings")
    ;
  match bindings_res {
    Ok(bindings) => {
      // Toggle in case we do or don't want the files emitted into version
      // control (or an easily readable location).  `true` will emit to the
      // build directory, and `false` will emit a source file to version control
      // under src.
      if false {
        // Write the bindings to the $OUT_DIR/bindings.rs file.
        let out_path = PathBuf::from(
          env::var("OUT_DIR").map_err(BuildError::EnvVarError)?,
        );
        bindings
          .write_to_file(out_path.join("bindings.rs"))
          .expect("Couldn't write bindings!");
      } else {
        // This gets a little more complex than using Bindings.write_to_file
        // because we need a prefix the file data to prevent rustfmt from
        // playing tug-of-war with the bindings.  The code is generated and so
        // we shouldn't fuss too much about the formatting, _or_ we should
        // format the code _after_ it's generated.  Perhaps in the future this
        // could be done programmatically to the buffer prior to writing it to
        // the file.
        let mut buffer = File::create("src/macos_bindings_emitted.rs")
          .map_err(BuildError::BindingFileWriteError)?;
        buffer.write(b"#[cfg_attr(rustfmt, rustfmt_skip)]\n\n")
          .map_err(BuildError::BindingFileWriteError)?;
        bindings
          .write(Box::new(&buffer))
          .expect("Couldn't write bindings!");
      }
    }
    Err(e) => {
      // We can continue because we've already written the bindings earlier.
      // The bindings should not be a breaker to building.
      error!("Got error {:?} but continuing anyways.", e);
    }
  };
  Ok(())
}

#[cfg(target_os = "macos")]
fn macos_binding_generate() -> Result<(), BuildError> {
  // Prevents this parsing issue that comes from three-entry typedefs in header
  // files:
  // https://github.com/rust-lang/rust-bindgen/issues/806
  // The error is "error: expected ';' after top level declarator".
  env::set_var("STD_CLANG_ARGS", "'${c_flags}'");
  // env::set_var("BINDGEN_EXTRA_CLANG_ARGS", "STD_CLANG_ARGS='${c_flags}'");

  let output = Command::new("xcrun")
    .arg("--sdk")
    .arg("macosx")
    .arg("--show-sdk-path")
    .output()
    .expect("Command failed to start.");
  let sdk_path = String::from_utf8(output.stdout)
    .map_err(BuildError::EncodeError)?
    .trim()
    // Prevent "creates a temporary value which is freed while still in use"
    // Rust idiom error when using trim() here.  Otherwise we have to create an
    // entirely separate variable, like an animal.
    .to_owned();
  let frameworks_path = format!("-F{}/System/Library/Frameworks", sdk_path);
  // Tell cargo to look for shared libraries in the specified directory
  println!("cargo:rustc-link-search={frameworks_path}");
  println!("bindgen:isysroot={sdk_path}");
  println!("cargo:rerun-if-changed=build.rs");
  // Tell cargo to invalidate the built crate whenever the wrapper changes.
  println!("cargo:rerun-if-changed=wrapper.h");
  println!("cargo:rerun-if-env-changed=BINDGEN_EXTRA_CLANG_ARGS");
  let _ = macos_framework_generate(&sdk_path, &frameworks_path)?;
  debug!("leave macos_binding_generate");
  Ok(())
}

fn main() -> Result<(), BuildError> {
  let mut logger = stderrlog::new();
  logger
    .verbosity(Level::Debug)
    .init()
    .inspect(|_| { debug!("Build logger initialized."); })
    .map_err(BuildError::LoggingInitializationError)
    ?;
  #[cfg(target_os = "macos")]
  macos_binding_generate()
    .inspect(|_| {
      info!("Emitting bindings successfully!");
    })
    ?;
  // To actually see the logs, uncomment this.
  // return Err(BuildError::ProveBuildRsIsRunningError());
  // #[ignore = "unreachable_code"]
  Ok(())
}
