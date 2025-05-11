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
    pub fn start() -> Result<()> {
        let env = Env::load()?;
        config::load(&env)?;
        let matches = Cli::command().get_matches();
        let cli = Cli::parse(&matches)?;
        let state = State::load(&env)?;
        let app = App { env, state };

        match cli.command {
            CliCommand::Enable(command) => command.run(app)?,
            CliCommand::Disable(command) => command.run(app)?,
            CliCommand::Update(command) => command.run(app)?,
            CliCommand::List(command) => command.run(app),
            CliCommand::Paths(command) => command.run(app)?,
            CliCommand::Hash(command) => command.run(app)?,
            CliCommand::Sync(command) => command.run(app)?,
            CliCommand::Run(command) => command.run(app),
            CliCommand::Alias(command) => command.run(app)?,
        }
        Ok(())
    }
}
