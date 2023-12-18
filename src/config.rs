use crate::error::AppError;
use clap::Parser;
use clap_verbosity_flag::Verbosity;
use std::env::var;

// Without a structopt declaration, the argument is positional.
#[derive(Debug, Parser)]
#[command(about = "Babysit your system with IFTTT automation.")]
pub struct CliConfig {
    #[arg(short, long)]
    pub sytters_path: Option<String>,
    #[command(flatten)]
    pub verbosity: Option<Verbosity>,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub sytters_path: String,
    pub verbosity: Verbosity,
  pub http_port: usize,
}

pub struct EnvConfig {
    pub sytters_path: Option<String>,
    pub verbosity: Option<Verbosity>,
}

// TODO: Remove this, since clap handles this now.
pub fn env_config_load() -> Result<EnvConfig, AppError> {
    let config = EnvConfig {
        sytters_path: var("sytter_sytters_path")
            .map_err(AppError::ConfigEnvVarError)
            .ok(),
        verbosity: var("sytter_verbosity")
            .map(|x| Verbosity::new(x.parse().unwrap(), 0))
            .map_err(AppError::ConfigEnvVarError)
            .ok(),
    };
    Ok(config)
}

pub fn config_cli_merge(
    env_config: EnvConfig,
    cli_config: CliConfig,
) -> Config {
    Config {
      // TODO: feed this to the HTTP server.
      http_port: 8080,
        sytters_path: cli_config
            .sytters_path
            .or(env_config.sytters_path)
            .unwrap_or("~/.config/sytter/sytters".to_string()),
        verbosity: cli_config
            .verbosity
            .or(env_config.verbosity)
            .unwrap_or(Verbosity::new(1, 0)),
    }
}

pub fn cli_parse() -> Result<CliConfig, AppError> {
    let cli = CliConfig::parse();
    Ok(cli)
}
