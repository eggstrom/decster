use anyhow::Result;

use crate::{
    cli::{Behavior, Cli, Command, InfoArgs},
    config::Config,
    paths::Paths,
    state::State,
};

pub struct App {
    behavior: Behavior,
    paths: Paths,
    config: Config,
}

impl App {
    pub fn run(cli: Cli) -> Result<()> {
        let paths = Paths::new(cli.config)?;
        let config = Config::parse(paths.config())?;
        let app = App {
            behavior: cli.behavior,
            paths,
            config,
        };

        match cli.command {
            Command::Info(args) => app.info(args),
            Command::Enable { modules } => app.enable(modules)?,
            Command::Disable { modules } => app.disable(modules),
            Command::Update { modules } => app.update(modules),
        }
        Ok(())
    }

    fn info(self, args: InfoArgs) {
        todo!()
    }

    fn enable(self, modules: Vec<String>) -> Result<()> {
        let state = State::new(self.paths.data())?;

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
                .enable(self.config.link_method, self.paths.data())?;
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
