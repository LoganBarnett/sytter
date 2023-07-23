#[derive(Debug)]
pub enum AppError {
    ConfigLoadError,
    ShellChildTerminatedError,
    ShellExecError((String, String)),
    ShellSpawnError(std::io::Error),
    ShellUtf8ConversionError(std::str::Utf8Error),
    TriggerInitializeError(String),
    TriggerAwaitError,
}
