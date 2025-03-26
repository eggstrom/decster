use std::collections::HashSet;

use anyhow::Result;

use crate::{
    cli::{Behavior, Cli, Command, InfoArgs},
    config::Config,
    paths,
    state::State,
};

pub struct App {
    behavior: Behavior,
    config: Config,
    state: State,
}
impl App {
    pub fn run(cli: Cli) -> Result<()> {
        paths::init()?;
        let config = Config::parse(cli.config.as_deref())?;
        let mut app = App {
            behavior: cli.behavior,
            config,
            state: State::load()?,
        };

        match cli.command {
            Command::Info(args) => app.info(args)?,
            Command::Enable { modules } => app.enable(modules.into_iter().collect())?,
            Command::Disable { modules } => app.disable(modules.into_iter().collect())?,
            Command::Update { modules } => app.update(modules.into_iter().collect()),
        }
        Ok(())
    }

    fn info(self, args: InfoArgs) -> Result<()> {
        todo!()
    }

    fn enable(&mut self, modules: HashSet<String>) -> Result<()> {
        if modules.is_empty() {
            self.config.enable_all_modules(&mut self.state);
        } else {
            for module in modules {
                self.config.enable_module(&mut self.state, &module);
            }
        }
        self.state.save()
    }

    fn disable(&mut self, modules: HashSet<String>) -> Result<()> {
        if modules.is_empty() {
            self.state.disable_all_modules();
        } else {
            for module in modules {
                self.state.disable_module(&module);
            }
        }
        self.state.save()
    }

    fn update(self, modules: Vec<String>) {
        todo!()
    }
}
