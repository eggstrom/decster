use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

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

#[derive(Args, Clone, Debug)]
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
    /// Ignore files that require elevated privileges
    #[arg(long, short, global = true)]
    pub ignore: bool,
    /// Skip modules with files that require elevated privileges
    #[arg(long, short, global = true)]
    pub skip: bool,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// List modules
    #[command(visible_alias = "l")]
    List(ListArgs),
    /// Check module state
    #[command(visible_alias = "c")]
    Check { modules: Vec<String> },
    /// Enable modules
    #[command(visible_aliases = ["e"])]
    Enable { modules: Vec<String> },
    /// Disable modules
    #[command(visible_alias = "d")]
    Disable { modules: Vec<String> },
    /// Update modules
    #[command(visible_alias = "u")]
    Update { modules: Vec<String> },
}

#[derive(Args, Clone, Debug)]
pub struct ListArgs {
    /// Only show enabled modules
    #[arg(long, short = 'E', conflicts_with = "disabled")]
    pub enabled: bool,
    /// Only show disabled modules
    #[arg(long, short = 'D', conflicts_with = "enabled")]
    pub disabled: bool,
}

impl ListArgs {
    pub fn filter(&self) -> ListFilter {
        match (self.enabled, self.disabled) {
            (false, false) => ListFilter::All,
            (true, false) => ListFilter::Enabled,
            (false, true) => ListFilter::Disabled,
            _ => unreachable!(),
        }
    }
}

pub enum ListFilter {
    All,
    Enabled,
    Disabled,
}
