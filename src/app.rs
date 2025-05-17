use anyhow::Result;

use crate::{
    cli::{Cli, CliCommand},
    config,
    env::Env,
    state::State,
};

pub struct App {
    pub env: Env,
    pub state: State,
}

impl App {
    pub fn run() -> Result<()> {
        let env = Env::load()?;
        config::load(&env)?;
        let matches = Cli::command(true).get_matches();
        let cli = Cli::parse(&matches);
        let state = State::load(&env)?;
        let app = App { env, state };
        app.run_command(cli.command)
    }

    fn run_command(self, command: CliCommand) -> Result<()> {
        match command {
            CliCommand::Enable(command) => command.run(self)?,
            CliCommand::Disable(command) => command.run(self)?,
            CliCommand::Update(command) => command.run(self)?,
            CliCommand::List(command) => command.run(self),
            CliCommand::Paths(command) => command.run(self)?,
            CliCommand::Hash(command) => command.run(self)?,
            CliCommand::Sync(command) => command.run(self)?,
            CliCommand::Run(command) => command.run(self),
            CliCommand::Alias(alias) => {
                let matches = alias.matches()?;
                let cli = Cli::parse(&matches);
                self.run_command(cli.command)?;
            }
        }
        Ok(())
    }
}
