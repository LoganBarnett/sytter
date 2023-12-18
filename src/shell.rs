use crate::error::AppError;
use log::*;
use std::process::Command;

// We could lazy_static this but eventually it'll become configurable.
pub fn shell_communication_functions() -> String {
  include_str!("shell-functions.sh").into()
}

pub fn shell_exec_check(
  http_port: usize,
  shell: &String,
  expected_exit_codes: &Vec<i32>,
  script: &String,
  id: &String,
) -> Result<bool, AppError> {
  let output = Command::new(shell)
    .args(["-c", script])
  // This requires curl, which isn't the most portable.  This is a rich location
  // for a better contribution.  Some possibilities:
  // 1. Use a Unix socket.  This would require writing a token if we were to
  // make it secure.  This might not be portable on Windows systems.
  // 2. Record the PID, and run Sytter again (or some derivative binary).  The
  //    binary can then communicate via HTTP or a socket as needed, with a
  //    preference towards HTTP for portability.
  // 3. Record the PID, Write to a state file, and send a kill signal to the
  //    Sytter PID in order to make Sytter reload the state file.  Of all of the
  //    options is is perhaps the least preferred.  We have some security,
  //    ownership, and cleanup concerns that would need to be addressed.
    .envs([
      ("sytter_token", shell_sytter_token()),
      ("sytter_port", http_port.to_string()),
    ])
    .output()
    .map_err(AppError::ShellSpawnError)?;
  let (stdout, stderr) =
    (from_utf8(&output.stdout)?, from_utf8(&output.stderr)?);
  debug!("{}", stdout);
  debug!("{}", stderr);
  output
    .status
    .code()
    .ok_or(AppError::ShellChildTerminatedError)
    .map(|code| expected_exit_codes.iter().any(|c| *c == code))
}

pub fn from_utf8(v: &Vec<u8>) -> Result<String, AppError> {
    std::str::from_utf8(v)
        .map(|s| s.to_string())
        .map_err(AppError::ShellUtf8ConversionError)
}

pub fn with_shell_functions(script: &String) -> String {
  format!(
    "{}\n{}",
    shell_communication_functions(),
    script,
  )
}

pub fn shell_exec_outputs(
  http_port: usize,
  shell: &String,
  script: &String,
  id: &String,
) -> Result<(String, String), AppError> {
  Command::new(shell)
    .args(["-c", script])
    .envs([
      ("sytter_token", shell_sytter_token()),
      ("sytter_port", http_port.to_string()),
    ])
    .output()
    .map_err(AppError::ShellSpawnError)
    .and_then(|output| {
      let outputs =
        (from_utf8(&output.stdout)?, from_utf8(&output.stderr)?);
      debug!("{}", outputs.0);
      debug!("{}", outputs.1);
      match output.status.code() {
        Some(status) => {
          if status == 0 {
            Ok(outputs)
          } else {
            Err(AppError::ShellExecError(outputs))
          }
        }
        None => Err(AppError::ShellChildTerminatedError),
      }
    })
}

// This should be a cryptographically generated token.
pub fn shell_sytter_token() -> String {
  "foobar".into()
}
