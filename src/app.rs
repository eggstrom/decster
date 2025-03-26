use std::collections::HashSet;

use anyhow::Result;
use crossterm::style::Stylize;

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
        let (names, filter) = args.modules();
        for (name, module) in self.config.modules(names, filter) {
            todo!()
        }
        Ok(())
    }

    fn enable(&mut self, modules: HashSet<String>) -> Result<()> {
        for (name, module) in self.config.modules(modules, ModuleFilter::All) {
            if self.state.is_module_enabled(name) {
                println!("Module {} is already enabled", name.magenta());
                continue;
            }

            println!("Adding sources for module {}", name.magenta());
            module.add_sources(&self.config, &mut self.state)?;
            println!("Enabling module {}", name.magenta());
            module.enable(&mut self.state, name)?;
        }
        self.state.save()?;
        Ok(())
    }

    fn disable(&mut self, modules: Vec<String>) -> Result<()> {
        for name in modules.iter().map(|string| string.as_str()) {
            if !self.state.is_module_enabled(name) {
                println!("Module {} isn't enabled", name.magenta());
                continue;
            }
            println!("Disabling module {}", name.magenta());
            self.state.remove_module(name)?;
        }
        self.state.save()?;
        Ok(())
    }

    fn update(self, modules: Vec<String>) {
        todo!()
    }
}
