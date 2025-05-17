use std::{env, iter};

use anyhow::{Result, anyhow};
use clap::{ArgMatches, Command, arg};
use itertools::Itertools;

use crate::config;

use super::Cli;

pub struct AliasCli<'a> {
    command: &'a str,
    args: Vec<&'a str>,
}

impl<'a> AliasCli<'a> {
    pub fn command<I>(alias: &'static str, command: I) -> Command
    where
        I: IntoIterator<Item = &'a str>,
    {
        Command::new(alias)
            .about(format!("Alias: {}", command.into_iter().join(" ")))
            .arg(arg!([ARGUMENTS]...).allow_hyphen_values(true))
    }

    pub fn parse(command: &'a str, matches: &'a ArgMatches) -> Self {
        let args = matches
            .get_many::<String>("ARGUMENTS")
            .unwrap_or_default()
            .map(|s| s.as_str())
            .collect();
        AliasCli { command, args }
    }

    pub fn matches(&self) -> Result<ArgMatches> {
        let program = env::args()
            .next()
            .ok_or(anyhow!("Couldn't get binary name"))?;
        let args = iter::once(program.as_str())
            .chain(config::alias(self.command)?)
            .chain(self.args.iter().copied());
        Ok(Cli::command().get_matches_from(args))
    }
}
