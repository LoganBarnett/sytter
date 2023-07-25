use clap::Parser;
use clap_verbosity_flag::Verbosity;
use crate::{
    error::AppError,
    logging::logger_init,
};

// Without a structopt declaration, the argument is positional.
#[derive(Debug, Parser)]
#[command(
    about = "Babysit your system with IFTTT automation.",
)]
pub struct Cli {
    #[command(flatten)]
    pub verbosity: Verbosity,
}


pub fn cli_parse() -> Result<Cli, AppError> {
    let cli = Cli::parse();
    logger_init(cli.verbosity.log_level())?;
    Ok(cli)
}
