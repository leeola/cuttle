pub mod cli;
pub mod validation;

use anyhow::Result;
use clap::Parser;
use cli::Cli;

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        cli::Commands::Validation(validation_cmd) => {
            validation::handle_command(validation_cmd).await?;
        }
    }

    Ok(())
}
