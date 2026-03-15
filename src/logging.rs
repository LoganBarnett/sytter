use crate::error::AppError;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn logger_init(log_level: Level) -> Result<(), AppError> {
  let level_filter = log_level;

  // Set up journald layer for systemd journal integration
  let journald_layer = tracing_journald::layer().map_err(|e| {
    AppError::LoggingInitializationError(
      format!("Failed to initialize journald layer: {}", e).into(),
    )
  })?;

  // Initialize the tracing subscriber with journald
  tracing_subscriber::registry()
    .with(journald_layer)
    .with(tracing_subscriber::filter::LevelFilter::from_level(
      level_filter,
    ))
    .try_init()
    .map_err(|e| {
      AppError::LoggingInitializationError(
        format!("Failed to initialize tracing subscriber: {}", e).into(),
      )
    })?;

  tracing::warn!("Setup tracing with level {}.", level_filter);
  Ok(())
}
