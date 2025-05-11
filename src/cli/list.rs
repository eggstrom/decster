use clap::Command;
use crossterm::style::Stylize;

use crate::{app::App, config};

pub struct ListCli;

impl ListCli {
    pub fn command() -> Command {
        Command::new("list").about("Show module definitions")
    }

    pub fn run(&self, app: App) {
        for (name, _) in config::modules() {
            let enabled = app.state.is_module_enabled(name);
            let state = match enabled {
                true => "Enabled".green(),
                false => "Disabled".red(),
            };
            println!("{} ({state})", name.magenta());
        }
    }
}
