use anyhow::{Result, bail};
use clap::{ArgMatches, Command, arg};
use crossterm::style::Stylize;

use crate::{app::App, config, globs::Globs, module::link::LinkMethod, utils::pretty::Pretty};

use super::common_args;

pub fn command() -> Command {
    Command::new("enable")
        .about("Enable modules")
        .args(common_args::link_method())
        .arg(arg!(<MODULES>...))
}

pub fn run(mut app: App, matches: ArgMatches) -> Result<()> {
    let method = LinkMethod::from_matches(&matches);
    let modules: Vec<_> = matches
        .get_many::<String>("MODULES")
        .unwrap()
        .map(|s| s.as_str())
        .collect();

    let globs = Globs::strict(&modules)?;
    let mut has_enabled = false;
    for (name, module) in config::modules_matching_globs(&globs) {
        let modules = module.import(name)?;
        if !app.state.is_module_enabled(name) {
            has_enabled = true;
            if let Err(err) = app.state.enable_module(&mut app.env, name, modules, method) {
                eprintln!("{} {err:?}", "error:".red());
            } else {
                println!("Enabled {}", name.magenta());
            }
        }
    }
    if !has_enabled {
        let modules = modules.as_slice();
        bail!("{} didn't match any disabled modules", modules.pretty());
    }
    app.state.save(&app.env)
}
