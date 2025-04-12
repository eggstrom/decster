use std::{path::PathBuf, str::FromStr};

use clap::{Args, Parser, Subcommand};

use crate::source::name::SourceName;

#[derive(Debug, Parser)]
pub struct Cli {
    #[group(flatten)]
    pub behavior: Behavior,
    #[command(subcommand)]
    pub command: Command,
    /// Set path to config directory
    #[arg(long, short, value_name = "PATH", global = true)]
    pub config: Option<PathBuf>,
}

#[derive(Args, Clone, Debug, Default)]
pub struct Behavior {
    /// Re-fetch sources
    #[arg(long, short, global = true)]
    pub fetch: bool,
    /// Overwrite existing files
    #[arg(long, short, global = true)]
    pub overwrite: bool,
    /// Show changes without doing anything
    #[arg(long, short, global = true)]
    pub dry_run: bool,
    /// Surpress output
    #[arg(long, short, global = true)]
    pub quiet: bool,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    #[command(alias = "e")]
    /// Enable modules
    Enable {
        #[arg(required = true, value_name = "PATTERNS")]
        modules: Vec<String>,
    },
    /// Disable modules
    #[command(alias = "d")]
    Disable {
        #[arg(required = true, value_name = "PATTERNS")]
        modules: Vec<String>,
    },
    /// Disable and re-enable modules
    #[command(alias = "u")]
    Update {
        #[arg(value_name = "PATTERNS")]
        modules: Vec<String>,
    },
    /// Show module definitions
    #[command(alias = "l")]
    List,
    /// Show owned paths
    #[command(alias = "p")]
    Paths,
    /// Show hashes of fetched sources
    #[command(alias = "h")]
    Hash {
        #[arg(value_parser = SourceName::from_str)]
        sources: Vec<SourceName>,
    },
}
