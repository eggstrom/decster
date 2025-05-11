use std::{io, os::unix::process::CommandExt, process};

use clap::{ArgMatches, Command, arg};

use crate::app::App;

pub struct RunCli<'a> {
    command: &'a str,
    args: Vec<&'a str>,
}

impl<'a> RunCli<'a> {
    pub fn command() -> Command {
        Command::new("run")
            .about("Run commands in config directory")
            .arg(arg!(<COMMAND>).allow_hyphen_values(true))
            .arg(arg!([ARGUMENTS]...).allow_hyphen_values(true))
    }

    pub fn parse(matches: &'a ArgMatches) -> Self {
        let command = matches.get_one::<String>("COMMAND").unwrap().as_str();
        let args = matches
            .get_many::<String>("ARGUMENTS")
            .unwrap_or_default()
            .map(|s| s.as_str())
            .collect();
        RunCli { command, args }
    }

    pub fn run(&self, app: App) {
        let mut command = process::Command::new(self.command);
        if !self.args.is_empty() {
            command.args(&self.args);
        }
        command
            .current_dir(app.env.config_dir())
            .stdout(io::stdout())
            .stderr(io::stderr());
        if let Some(err) = command.exec().raw_os_error() {
            process::exit(err);
        }
    }
}
