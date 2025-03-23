use anyhow::Result;
use crossterm::style::Stylize;
use log::info;

use crate::{
    cli::{Behavior, Cli, Command, InfoArgs},
    config::Config,
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
            Command::Enable { modules } => app.enable(modules)?,
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

    fn enable(&mut self, modules: Vec<String>) -> Result<()> {
        for name in modules.iter() {
            for link in self.config.links(&name)? {
                let name = link.source_name();
                let source = self.config.source(name)?;
                self.state.add_source(name, source)?;
            }
        }

        for module in modules.iter().map(|s| s.as_str()) {
            info!("Enabling module: {}", module.magenta());
            self.config
                .module(module)?
                .enable(self.config.link_method, &mut self.state)?;
        }
        self.state.save()?;
        Ok(())
    }

    fn disable(&mut self, modules: Vec<String>) -> Result<()> {
        for module in modules.iter().map(|s| s.as_str()) {
            info!("Disabling module: {}", module.magenta());
            self.config
                .module(module)?
                .disable(self.config.link_method, &mut self.state)?;
        }
        self.state.save()?;
        Ok(())
    }

    fn update(self, modules: Vec<String>) {
        todo!()
    }
}
