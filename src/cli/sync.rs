use anyhow::Result;
use clap::{ArgMatches, Command, arg};

use crate::app::App;

pub fn command() -> Command {
    Command::new("sync")
        .about("Update system packages to match enabled modules")
        .arg(arg!(-i --install "Install without uninstalling").conflicts_with("uninstall"))
        .arg(arg!(-u --uninstall "Uninstall without installing"))
}

pub fn run(app: App, matches: ArgMatches) -> Result<()> {
    let (install, uninstall) = match (matches.get_flag("install"), matches.get_flag("uninstall")) {
        (false, false) => (true, true),
        (install, uninstall) => (install, uninstall),
    };

    for (manager, packages) in app.state.packages() {
        manager.sync(install, uninstall, &packages)?;
    }
    Ok(())
}
