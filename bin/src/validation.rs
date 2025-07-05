pub mod baseline;
pub mod diff;
pub mod run;
pub mod suite;

use crate::cli::{ValidationCommand, ValidationSubcommands};
use anyhow::Result;

pub async fn handle_command(cmd: ValidationCommand) -> Result<()> {
    match cmd.command {
        ValidationSubcommands::Run {
            name,
            output,
            compare_baseline,
            timeout,
        } => run::run_validations(name, output, compare_baseline, timeout).await,
        ValidationSubcommands::List => {
            suite::list_validations();
            Ok(())
        }
        ValidationSubcommands::Diff {
            baseline,
            current,
            format,
            output,
        } => diff::compare_states(baseline, current, format, output).await,
        ValidationSubcommands::Baseline { command } => {
            baseline::handle_baseline_command(command).await
        }
    }
}
