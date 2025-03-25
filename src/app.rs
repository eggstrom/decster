use std::collections::HashSet;

use anyhow::Result;
use crossterm::style::Stylize;
use log::{error, info};

use crate::{
    cli::{Behavior, Cli, Command, InfoArgs},
    config::Config,
    module::ModuleFilter,
    state::State,
};

pub struct App {
    behavior: Behavior,
    config: Config,
    state: State,
}

impl App {
    pub fn run(cli: Cli) -> Result<()> {
        let config = Config::parse(cli.config.as_deref())?;
        let mut app = App {
            behavior: cli.behavior,
            config,
            state: State::load()?,
        };

        match cli.command {
            Command::Info(args) => app.info(args)?,
            Command::Enable { modules } => app.enable(modules.into_iter().collect())?,
            Command::Disable { modules } => app.disable(modules)?,
            Command::Update { modules } => app.update(modules),
        }
        Ok(())
    }

    fn info(self, args: InfoArgs) -> Result<()> {
        if args.owned_files {
            println!("{}", self.state);
            return Ok(());
        }

        let (names, filter) = args.modules();
        for (name, module) in self.config.modules(names, filter) {
            todo!()
        }
        Ok(())
    }

    fn enable(&mut self, modules: HashSet<String>) -> Result<()> {
        for (name, module) in self.config.modules(modules, ModuleFilter::All) {
            info!("Adding sources for {}", name.magenta());
            match module.add_sources(&self.config, &mut self.state) {
                Ok(()) => {
                    info!("Enabling {}", name.magenta());
                    module.enable(&mut self.state, name, self.config.link_method);
                }
                Err(error) => error!("{error:?}"),
            }
        }
        self.state.save()?;
        Ok(())
    }

    fn disable(&mut self, modules: Vec<String>) -> Result<()> {
        for name in modules.iter() {
            if let Err(error) = self.state.remove_module(name) {
                error!("{error:?}");
            }
        }
        self.state.save()?;
        Ok(())
    }

    fn update(self, modules: Vec<String>) {
        todo!()
    }
}
