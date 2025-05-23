use anyhow::{Result, bail};
use clap::{ArgMatches, Command, arg};
use crossterm::style::Stylize;

use crate::{app::App, globs::Globs, utils::pretty::Pretty};

pub fn command() -> Command {
    Command::new("disable")
        .about("Disable modules")
        .arg(arg!(<MODULES>...))
}

pub fn run(mut app: App, matches: ArgMatches) -> Result<()> {
    let modules: Vec<_> = matches
        .get_many::<String>("MODULES")
        .unwrap()
        .map(|s| s.as_str())
        .collect();

    let globs = Globs::strict(&modules)?;
    let matches = app.state.module_names_matching_globs(&globs);
    if matches.is_empty() {
        let modules = modules.as_slice();
        bail!("{} didn't match any enabled modules", modules.pretty());
    }
    for module in matches {
        if let Err(err) = app.state.disable_module(&app.env, &module) {
            eprintln!("{} {err:?}", "error:".red());
        } else {
            println!("Disabled {}", module.magenta());
        }
    }
    app.state.save(&app.env)
}
