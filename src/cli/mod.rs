use anyhow::{Result, bail};
use clap::{ArgMatches, Command, arg, command};

use crate::{app::App, config};

pub mod alias;
pub mod disable;
pub mod enable;
pub mod run;
pub mod show;
pub mod sync;
pub mod update;

pub fn command() -> Command {
    command!()
        .arg_required_else_help(true)
        .arg(arg!(-f --fetch "Re-fetch sources").global(true))
        .subcommand(enable::command())
        .subcommand(disable::command())
        .subcommand(update::command())
        .subcommand(sync::command())
        .subcommand(show::command())
        .subcommand(run::command())
}

pub fn command_with_aliases() -> Result<Command> {
    let mut cli = command();
    for (name, command) in config::aliases() {
        if cli.find_subcommand(name).is_some() {
            bail!("Couldn't overwrite command `{name}` with an alias");
        }
        cli = cli.subcommand(alias::command(name, command));
    }
    Ok(cli)
}

pub fn run(app: App) -> Result<()> {
    let matches = command_with_aliases()?.get_matches();
    let fetch = matches.get_flag("fetch");
    run_inner(app, matches)
}

fn run_inner(app: App, mut matches: ArgMatches) -> Result<()> {
    let (subcommand, matches) = matches.remove_subcommand().unwrap();
    match subcommand.as_str() {
        "enable" => enable::run(app, matches)?,
        "disable" => disable::run(app, matches)?,
        "update" => update::run(app, matches)?,
        "sync" => sync::run(app, matches)?,
        "show" => show::run(app, matches)?,
        "run" => run::run(app, matches),
        _ => run_inner(app, alias::matches(&subcommand, matches)?)?,
    }
    Ok(())
}
