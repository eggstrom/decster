use std::{env, iter};

use anyhow::{Result, anyhow};
use clap::{ArgMatches, Command, arg};
use itertools::Itertools;

use crate::config;

pub fn command<'a, I>(alias: &'static str, command: I) -> Command
where
    I: IntoIterator<Item = &'a str>,
{
    Command::new(alias)
        .about(format!("Alias: {}", command.into_iter().join(" ")))
        .arg(arg!([ARGUMENTS]...).allow_hyphen_values(true))
}

pub fn matches(command: &str, matches: ArgMatches) -> Result<ArgMatches> {
    let args: Vec<_> = matches
        .get_many::<String>("ARGUMENTS")
        .unwrap_or_default()
        .map(|s| s.as_str())
        .collect();

    let program = env::args()
        .next()
        .ok_or(anyhow!("Couldn't get name of binary"))?;
    let args = iter::once(program.as_str())
        .chain(config::alias(command)?)
        .chain(args.iter().copied());
    Ok(super::command().get_matches_from(args))
}
