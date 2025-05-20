use alias::AliasCli;
use anyhow::{Result, bail};
use clap::{ArgMatches, Command, arg, command};
use disable::DisableCli;
use enable::EnableCli;
use run::RunCli;
use show::ShowCli;
use sync::SyncCli;
use update::UpdateCli;

use crate::config;

pub mod alias;
pub mod disable;
pub mod enable;
pub mod run;
pub mod show;
pub mod sync;
pub mod update;

pub struct Cli<'a> {
    pub fetch: bool,
    pub command: CliCommand<'a>,
}

impl<'a> Cli<'a> {
    pub fn command() -> Command {
        command!()
            .arg_required_else_help(true)
            .arg(arg!(-f --fetch "Re-fetch sources").global(true))
            .subcommand(EnableCli::command())
            .subcommand(DisableCli::command())
            .subcommand(UpdateCli::command())
            .subcommand(SyncCli::command())
            .subcommand(ShowCli::command())
            .subcommand(RunCli::command())
    }

    pub fn command_with_aliases() -> Result<Command> {
        let mut cli = Self::command();
        for (name, command) in config::aliases() {
            if cli.find_subcommand(name).is_some() {
                bail!("Couldn't overwrite command `{name}` with an alias");
            }
            cli = cli.subcommand(AliasCli::command(name, command));
        }
        Ok(cli)
    }

    pub fn parse(matches: &'a ArgMatches) -> Self {
        let Some((subcommand, matches)) = matches.subcommand() else {
            unreachable!()
        };
        Cli {
            fetch: matches.get_flag("fetch"),
            command: match subcommand {
                "enable" => CliCommand::Enable(EnableCli::parse(matches)),
                "disable" => CliCommand::Disable(DisableCli::parse(matches)),
                "update" => CliCommand::Update(UpdateCli::parse(matches)),
                "sync" => CliCommand::Sync(SyncCli::parse(matches)),
                "show" => CliCommand::Show(ShowCli::parse(matches)),
                "run" => CliCommand::Run(RunCli::parse(matches)),
                _ => CliCommand::Alias(AliasCli::parse(subcommand, matches)),
            },
        }
    }
}

pub enum CliCommand<'a> {
    Enable(EnableCli<'a>),
    Disable(DisableCli<'a>),
    Update(UpdateCli<'a>),
    Sync(SyncCli),
    Show(ShowCli<'a>),
    Run(RunCli<'a>),
    Alias(AliasCli<'a>),
}
