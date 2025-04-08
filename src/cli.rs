use std::{collections::HashSet, path::PathBuf, str::FromStr};

use clap::{Args, Parser, Subcommand};

use crate::{source::name::SourceName, state::ModuleFilter};

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
    /// Show hashes of fetched sources
    #[command(alias = "h")]
    Hash {
        #[arg(value_parser = SourceName::from_str)]
        sources: Vec<SourceName>,
    },
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
