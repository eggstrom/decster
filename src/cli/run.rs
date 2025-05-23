use std::{io, os::unix::process::CommandExt, process};

use clap::{ArgMatches, Command, arg};

use crate::app::App;

pub fn command() -> Command {
    Command::new("run")
        .about("Run commands in config directory")
        .arg(arg!(<COMMAND>).allow_hyphen_values(true))
        .arg(arg!([ARGUMENTS]...).allow_hyphen_values(true))
}

pub fn run(app: App, matches: ArgMatches) {
    let command = matches.get_one::<String>("COMMAND").unwrap().as_str();
    let args: Vec<_> = matches
        .get_many::<String>("ARGUMENTS")
        .unwrap_or_default()
        .map(|s| s.as_str())
        .collect();

    let mut command = process::Command::new(command);
    if !args.is_empty() {
        command.args(&args);
    }
    command
        .current_dir(app.env.config_dir())
        .stdout(io::stdout())
        .stderr(io::stderr());
    if let Some(err) = command.exec().raw_os_error() {
        process::exit(err);
    }
}
