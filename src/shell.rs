use crate::error::AppError;
use std::process::Command;

pub fn shell_exec_check(
    shell: &String,
    expected_exit_codes: &Vec<i32>,
    script: &String,
) -> Result<bool, AppError> {
    let output = Command::new(shell)
        .args(["-c", script])
        .output()
        .map_err(AppError::ShellSpawnError)?;
    // TODO: debug-log the shell outputs.
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

pub fn shell_exec_outputs(
    shell: &String,
    script: &String,
) -> Result<(String, String), AppError> {
    Command::new(shell)
        .args(["-c", script])
        .output()
        .map_err(AppError::ShellSpawnError)
        .and_then(|output| {
            let outputs =
                (from_utf8(&output.stdout)?, from_utf8(&output.stderr)?);
            // TODO: debug-log the shell outputs.
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
