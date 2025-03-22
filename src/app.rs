use anyhow::Result;

use crate::{
    cli::{Behavior, Cli, Command, InfoArgs},
    config::Config,
    state::State,
};

pub struct App {
    behavior: Behavior,
    config: Config,
}

impl App {
    pub fn run(cli: Cli) -> Result<()> {
        let config = Config::parse(cli.config.as_deref())?;
        let app = App {
            behavior: cli.behavior,
            config,
        };

        match cli.command {
            Command::Info(args) => app.info(args)?,
            Command::Enable { modules } => app.enable(modules)?,
            Command::Disable { modules } => app.disable(modules),
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

    fn enable(self, modules: Vec<String>) -> Result<()> {
        let state = State::new()?;

        let builder = state.source_builder()?;
        for module in modules.iter() {
            for link in self.config.links(&module)? {
                let name = link.source_name();
                let source = self.config.source(name)?;
                builder.add_source(name, source)?;
            }
        }
        builder.save()?;

        for module in modules.iter() {
            self.config
                .module(module)?
                .enable(self.config.link_method)?;
        }
        Ok(())
    }

    fn disable(self, modules: Vec<String>) {
        todo!()
    }

    fn update(self, modules: Vec<String>) {
        todo!()
    }
}
