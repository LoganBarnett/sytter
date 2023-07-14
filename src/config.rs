use crate::error;
use error::AppError;

pub struct Config {
    pub sytters_path: String,
}

pub fn config_load() -> Result<Config, AppError> {
    let config = Config { sytters_path: "".to_string() };
    Ok(config)
}
