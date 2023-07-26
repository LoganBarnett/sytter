use std::env::VarError;

#[derive(Debug)]
pub enum AppError {
    ConfigEnvVarError(VarError),
    LoggingInitializationError(log::SetLoggerError),
    ShellChildTerminatedError,
    ShellExecError((String, String)),
    ShellSpawnError(std::io::Error),
    ShellUtf8ConversionError(std::str::Utf8Error),
    TriggerInitializeError(String),
}
