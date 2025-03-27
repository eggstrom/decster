use std::collections::HashSet;

use anyhow::Result;
use clap::Parser;

use crate::{
    cli::{Cli, Command, InfoArgs},
    global,
    state::State,
};

pub struct App {
    state: State,
}

impl App {
    pub fn run() -> Result<()> {
        let cli = Cli::parse();
        global::init(&cli)?;
        let mut app = App {
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
            self.state.enable_all_modules();
        } else {
            for module in modules {
                self.state.enable_module(&module);
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
