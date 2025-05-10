use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Cli {
    /// Set path to config directory
    #[arg(long, short, value_name = "PATH", global = true)]
    pub config: Option<PathBuf>,
    #[group(flatten)]
    pub behavior: Behavior,
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Args, Clone, Debug, Default)]
pub struct Behavior {
    /// Re-fetch sources
    #[arg(long, short, global = true)]
    pub fetch: bool,
}

#[derive(Clone, Debug, Subcommand)]
pub enum CliCommand {
    #[command()]
    /// Enable modules
    Enable {
        #[arg(required = true)]
        modules: Vec<String>,
    },
    /// Disable modules
    #[command()]
    Disable {
        #[arg(required = true)]
        modules: Vec<String>,
    },
    /// Disable and re-enable modules
    #[command()]
    Update { modules: Vec<String> },
    /// Show module definitions
    #[command()]
    List,
    /// Show owned paths
    #[command()]
    Paths,
    /// Show hashes of fetched sources
    #[command()]
    Hash { sources: Vec<String> },
    /// Update system packages to match enabled modules
    Sync(SyncArgs),
    /// Run Git commands in config directory
    #[command()]
    Git {
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Args, Clone, Debug)]
pub struct SyncArgs {
    /// Install without uninstalling
    #[arg(long, short, conflicts_with = "uninstall")]
    install: bool,
    /// Uninstall without installing
    #[arg(long, short)]
    uninstall: bool,
}

impl SyncArgs {
    pub fn should_install_and_uninstall(&self) -> (bool, bool) {
        match (self.install, self.uninstall) {
            (false, false) => (true, true),
            (install, uninstall) => (install, uninstall),
        }
    }
}
