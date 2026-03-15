use crate::error::AppError;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn logger_init(log_level: Level) -> Result<(), AppError> {
  let level_filter = log_level;

  // Try to set up journald layer for systemd journal integration
  // Fall back to stderr if journald is not available (e.g., on macOS or in tests)
  match tracing_journald::layer() {
    Ok(journald_layer) => {
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
      tracing::warn!("Setup tracing with journald at level {}.", level_filter);
    }
    Err(_) => {
      // Journald not available, use stderr instead
      use tracing_subscriber::fmt;
      tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr).with_ansi(true))
        .with(tracing_subscriber::filter::LevelFilter::from_level(
          level_filter,
        ))
        .try_init()
        .map_err(|e| {
          AppError::LoggingInitializationError(
            format!("Failed to initialize tracing subscriber: {}", e).into(),
          )
        })?;
      tracing::warn!(
        "Journald not available, using stderr. Log level: {}.",
        level_filter
      );
    }
  }

  Ok(())
}
