use std::{collections::HashSet, path::PathBuf};

use clap::{Args, Parser, Subcommand};

use crate::state::ModuleFilter;

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
    /// Display information
    #[command(alias = "i")]
    Info(InfoArgs),
    /// Enable modules
    #[command(alias = "e")]
    Enable { modules: Vec<String> },
    /// Disable modules
    #[command(alias = "d")]
    Disable { modules: Vec<String> },
    /// Disable and re-enable modules
    #[command(alias = "u")]
    Update { modules: Vec<String> },
}

#[derive(Args, Clone, Debug)]
pub struct InfoArgs {
    pub modules: Vec<String>,
    /// Only show enabled modules
    #[arg(long, short = 'E', conflicts_with = "disabled")]
    pub enabled: bool,
    /// Only show disabled modules
    #[arg(long, short = 'D')]
    pub disabled: bool,
}

impl InfoArgs {
    pub fn modules(self) -> (HashSet<String>, ModuleFilter) {
        (
            self.modules.into_iter().collect(),
            match () {
                _ if self.enabled => ModuleFilter::Enabled,
                _ if self.disabled => ModuleFilter::Disabled,
                _ => ModuleFilter::All,
            },
        )
    }
}
