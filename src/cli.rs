use std::{collections::HashSet, path::PathBuf};

use clap::{Args, Parser, Subcommand};

use crate::module::ModuleFilter;

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
    #[group(flatten)]
    pub behavior: Behavior,
    /// Set path to config directory
    #[arg(long, short, value_name = "PATH", global = true)]
    pub config: Option<PathBuf>,
}

#[derive(Args, Clone, Debug, Default)]
pub struct Behavior {
    /// Surpress output
    #[arg(long, short, global = true)]
    pub quiet: bool,
    /// Show changes without doing anything
    #[arg(long, short, global = true)]
    pub dry_run: bool,
    /// Overwrite existing files
    #[arg(long, short, global = true)]
    pub force: bool,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Display information about modules
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
    /// Show owned files
    #[arg(long, short)]
    pub owned_files: bool,
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
