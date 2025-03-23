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
            Command::Disable { modules } => app.disable(modules.into_iter().collect())?,
            Command::Update { modules } => app.update(modules.into_iter().collect()),
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
            let unwritable_paths = module.unwritable_paths(&self.state);
            if !unwritable_paths.is_empty() {
                error!(
                    "Can't enable {} because the following paths aren't writable:",
                    name.magenta()
                );
                for path in unwritable_paths {
                    println!("  {}", path.display());
                }
                continue;
            }

            info!("Adding sources for {}", name.magenta());
            match module.add_sources(&self.config, &mut self.state) {
                Ok(()) => {
                    info!("Enabling {}", name.magenta());
                    module.enable(self.config.link_method, &mut self.state);
                }
                Err(error) => error!("{error:?}"),
            }
        }
        self.state.save()?;
        Ok(())
    }

    fn disable(&mut self, modules: HashSet<String>) -> Result<()> {
        for (name, module) in self.config.modules(modules, ModuleFilter::All) {
            info!("Disabling {}", name.magenta());
            module.disable(&mut self.state);
        }
        self.state.save()?;
        Ok(())
    }

    fn update(self, modules: Vec<String>) {
        todo!()
    }
}
