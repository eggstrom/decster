use anyhow::{Result, bail};
use clap::{ArgMatches, Command, arg};
use crossterm::style::Stylize;

use crate::{app::App, config, globs::Globs, module::link::LinkMethod, utils::pretty::Pretty};

use super::common_args;

pub fn command() -> Command {
    Command::new("update")
        .about("Disable and re-enable modules")
        .args(common_args::link_method())
        .arg(arg!([MODULES]...))
}

pub fn run(mut app: App, matches: ArgMatches) -> Result<()> {
    let method = LinkMethod::from_matches(&matches);
    let modules: Vec<_> = matches
        .get_many::<String>("MODULES")
        .unwrap_or_default()
        .map(|s| s.as_str())
        .collect();

    if modules.is_empty() {
        let modules = app.state.module_names();
        match run_inner(&mut app, &modules, method) {
            Ok(false) => bail!("There are no enabled modules"),
            Err(err) => eprintln!("{} {err:?}", "error:".red()),
            _ => (),
        }
    } else {
        let globs = Globs::permissive(&modules)?;
        let modules = app.state.module_names_matching_globs(&globs);
        match run_inner(&mut app, &modules, method) {
            Ok(false) => {
                let modules = modules.as_slice();
                bail!("{} didn't match any enabled modules", modules.pretty());
            }
            Err(err) => eprintln!("{} {err}", "error:".red()),
            _ => (),
        }
    }
    app.state.save(&app.env)
}

fn run_inner(app: &mut App, modules: &[String], method: LinkMethod) -> Result<bool> {
    let mut has_updated = false;
    for name in modules {
        has_updated = true;
        let name = name.as_ref();
        let modules = config::module(name).map(|(_, module)| module.import(name));
        app.state
            .update_module(&mut app.env, name, modules.transpose()?, method)?;
        println!("Updated {}", name.magenta());
    }
    Ok(has_updated)
}
