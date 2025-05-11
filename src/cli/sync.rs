use anyhow::Result;
use clap::{ArgMatches, Command, arg};

use crate::app::App;

pub struct SyncCli {
    install: bool,
    uninstall: bool,
}

impl SyncCli {
    pub fn command() -> Command {
        Command::new("sync")
            .about("Update system packages to match enabled modules")
            .arg(arg!(-i --install "Install without uninstalling").conflicts_with("uninstall"))
            .arg(arg!(-u --uninstall "Uninstall without installing"))
    }

    pub fn parse(matches: &ArgMatches) -> Self {
        let install = matches.get_flag("install");
        let uninstall = matches.get_flag("uninstall");
        SyncCli { install, uninstall }
    }

    pub fn run(&self, app: App) -> Result<()> {
        let (install, uninstall) = match (self.install, self.uninstall) {
            (false, false) => (true, true),
            (install, uninstall) => (install, uninstall),
        };
        for (manager, packages) in app.state.packages() {
            manager.sync(install, uninstall, &packages)?;
        }
        Ok(())
    }
}
