#[derive(Debug)]
pub enum AppError {
    ConfigLoadError,
    TriggerInitializeError(String),
    TriggerAwaitError,
}
