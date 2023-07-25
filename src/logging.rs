use crate::error::AppError;
use log::{warn, Level};

pub fn logger_init(verbosity: Option<Level>) -> Result<(), AppError> {
    match verbosity {
        Some(level) => {
            let mut logger = stderrlog::new();
            logger
                .verbosity(level)
                .init()
                .map_err(AppError::LoggingInitializationError)
                ?;
            warn!("Setup logger with verbosity {}.", level);
            Ok(())
        },
        None => Ok(()),
    }
}
