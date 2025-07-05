use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cuttle")]
#[command(about = "Cuttle - Blender automation and testing tool")]
#[command(long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Blender state validation harness
    Validation(ValidationCommand),
}

#[derive(Parser)]
pub struct ValidationCommand {
    #[command(subcommand)]
    pub command: ValidationSubcommands,
}

#[derive(Subcommand)]
pub enum ValidationSubcommands {
    /// Run validations and capture Blender state
    Run {
        /// Name of specific validation to run (runs all if not specified)
        name: Option<String>,

        /// Output directory for validation results
        #[arg(short, long, default_value = "validation_results")]
        output: PathBuf,

        /// Compare against baseline after running
        #[arg(short, long)]
        compare_baseline: bool,

        /// Timeout for each validation in seconds
        #[arg(long, default_value = "30")]
        timeout: u64,
    },

    /// List available validations
    List,

    /// Compare Blender states
    Diff {
        /// First state file to compare
        baseline: PathBuf,

        /// Second state file to compare
        current: PathBuf,

        /// Output format (json, yaml, text)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Manage baseline state
    Baseline {
        #[command(subcommand)]
        command: BaselineCommands,
    },
}

#[derive(Subcommand)]
pub enum BaselineCommands {
    /// Set new baseline from current state
    Set {
        /// Source state file
        source: PathBuf,

        /// Baseline name
        #[arg(short, long, default_value = "default")]
        name: String,
    },

    /// List available baselines
    List,

    /// Show baseline details
    Show {
        /// Baseline name
        #[arg(default_value = "default")]
        name: String,
    },

    /// Remove baseline
    Remove {
        /// Baseline name
        name: String,
    },
}
