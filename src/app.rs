use anyhow::Result;
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
        info!("Enabling modules: {:?}", modules);

        let builder = self.state.source_builder()?;
        for module in modules.iter() {
            for link in self.config.links(&module)? {
                let name = link.source_name();
                let source = self.config.source(name)?;
                builder.add_source(name, source)?;
            }
        }
        builder.save()?;

        for module in modules.iter() {
            info!("Enabling module `{module}`");
            self.config
                .module(module)?
                .enable(self.config.link_method, &mut self.state)?;
        }
        self.state.save()?;
        Ok(())
    }

    fn disable(&mut self, modules: Vec<String>) -> Result<()> {
        for module in modules.iter() {
            info!("Disabling module `{module}`");
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
