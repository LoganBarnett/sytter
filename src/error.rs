use std::env::VarError;

#[derive(Debug)]
pub enum AppError {
    ConfigEnvVarError(VarError),
    LoggingInitializationError(log::SetLoggerError),
    PowerHookRegistrationFailed,
    PowerEventParseError,
    PowerEventsMissingError,
    ShellChildTerminatedError,
    ShellExecError((String, String)),
    ShellSpawnError(std::io::Error),
    ShellUtf8ConversionError(std::str::Utf8Error),
    SytterDeserializeError(toml::de::Error),
    SytterDeserializeRawError(String),
    SytterMissingComponentError(String),
    SytterReadError(std::io::Error),
    SyttersDirInvalidError(std::io::Error),
    TriggerInitializeError(String),
}
