use crate::error::AppError;
use clap::Parser;
use std::env::var;
use tracing::Level;

// Without a structopt declaration, the argument is positional.
#[derive(Debug, Parser)]
#[command(about = "Babysit your system with IFTTT automation.")]
pub struct CliConfig {
  #[arg(short, long)]
  pub sytters_path: Option<String>,
  #[arg(short, long, help = "Log level: trace, debug, info, warn, error")]
  pub log_level: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Config {
  pub sytters_path: String,
  pub log_level: Level,
  pub http_port: usize,
}

pub struct EnvConfig {
  pub sytters_path: Option<String>,
  pub log_level: Option<String>,
  pub http_port: Option<usize>,
}

// TODO: Remove this, since clap handles this now.
pub fn env_config_load() -> Result<EnvConfig, AppError> {
  let config = EnvConfig {
    sytters_path: var("sytter_sytters_path")
      .map_err(AppError::ConfigEnvVarError)
      .ok(),
    log_level: var("sytter_log_level")
      .map_err(AppError::ConfigEnvVarError)
      .ok(),
    http_port: var("sytter_http_port").ok().and_then(|s| s.parse().ok()),
  };
  Ok(config)
}

fn parse_log_level(level_str: &str) -> Result<Level, AppError> {
  match level_str.to_lowercase().as_str() {
    "trace" => Ok(Level::TRACE),
    "debug" => Ok(Level::DEBUG),
    "info" => Ok(Level::INFO),
    "warn" => Ok(Level::WARN),
    "error" => Ok(Level::ERROR),
    _ => Err(AppError::ConfigInvalidLogLevel(level_str.to_string())),
  }
}

pub fn config_cli_merge(
  env_config: EnvConfig,
  cli_config: CliConfig,
) -> Result<Config, AppError> {
  let log_level_str = cli_config
    .log_level
    .or(env_config.log_level)
    .unwrap_or("info".to_string());

  Ok(Config {
    http_port: env_config.http_port.unwrap_or(8080),
    sytters_path: cli_config
      .sytters_path
      .or(env_config.sytters_path)
      .unwrap_or("~/.config/sytter/sytters".to_string()),
    log_level: parse_log_level(&log_level_str)?,
  })
}

pub fn cli_parse() -> Result<CliConfig, AppError> {
  let cli = CliConfig::parse();
  Ok(cli)
}
