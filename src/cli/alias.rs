use anyhow::Result;
use clap::{ArgMatches, Command, arg};

use crate::app::App;

pub struct AliasCli<'a> {
    args: Vec<&'a str>,
}

impl<'a> AliasCli<'a> {
    pub fn command(alias: &'static str, command: &str) -> Command {
        Command::new(alias)
            .about(format!("Alias: {command}"))
            .arg(arg!([ARGUMENTS]...).allow_hyphen_values(true))
    }

    pub fn parse(matches: &'a ArgMatches) -> Self {
        let args = matches
            .get_many::<String>("ARGUMENTS")
            .unwrap_or_default()
            .map(|s| s.as_str())
            .collect();
        AliasCli { args }
    }

    pub fn run(&self, app: App) -> Result<()> {
        todo!();
    }
}
