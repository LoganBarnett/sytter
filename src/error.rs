use std::env::VarError;

use actix_web::ResponseError;
use strum_macros::Display;

#[derive(Debug, Display)]
pub enum AppError {
    ConfigEnvVarError(VarError),
    DeviceConnectionEventsMissingError(),
    DeviceConnectionEventsParseError(),
    DeviceConnectionEventParseError(),
    EventMutexLockError(String),
  HttpBindError(std::io::Error),
  HttpHeaderValueToStringError(actix_web::http::header::ToStrError),
  HttpJsonSerializeError(serdeconv::Error),
  HttpStartError(std::io::Error),
    ListenerRegistrationFailed,
    LoggingInitializationError(log::SetLoggerError),
    KernelPortCallbackNotFoundError(usize),
    MachPortRegistrationFailed(),
    PowerHookRegistrationFailed,
    PowerEventParseError,
    PowerEventsMissingError,
    ShellChildTerminatedError,
    ShellExecError((String, String)),
    ShellSpawnError(std::io::Error),
    ShellUtf8ConversionError(std::str::Utf8Error),
  StateMutexPoisonedError(),
    SytterDeserializeError(toml::de::Error),
    SytterDeserializeRawError(String),
    SytterMissingComponentError(String),
    SytterReadError(std::io::Error),
    SyttersDirInvalidError(std::io::Error),
    TriggerInitializeError(String),
  TriggerRuntimeError(String),
  TriggersMissing(String),
    // SytterVariableUpdateError(SytterVariable),
}

impl ResponseError for AppError {

}
