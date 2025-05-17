use anyhow::Result;
use clap::Command;
use crossterm::style::Stylize;

use crate::{app::App, utils::pretty::Pretty};

pub struct PathsCli;

impl PathsCli {
    pub fn command() -> Command {
        Command::new("paths").about("Show owned paths")
    }

    pub fn run(&self, app: App) -> Result<()> {
        let owned_paths = app.state.owned_paths();
        if owned_paths.len() == 0 {
            println!("There are no owned paths");
        }
        for (module, paths) in owned_paths {
            println!("Paths owned by module {}:", module.magenta());
            for (path, info) in paths {
                println!("  {} ({})", app.env.tildefy(path).pretty(), info.kind());
            }
        }
        Ok(())
    }
}
