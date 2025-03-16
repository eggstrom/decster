use anyhow::Result;

use crate::{
    cli::{Behavior, Cli, Command, InfoArgs},
    config::Config,
    paths::Paths,
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
            Command::Enable { modules } => app.enable(modules),
            Command::Disable { modules } => app.disable(modules),
            Command::Update { modules } => app.update(modules),
        }
        Ok(())
    }

    fn info(self, args: InfoArgs) {
        todo!()
    }

    fn enable(self, modules: Vec<String>) {
        todo!()
    }

    fn disable(self, modules: Vec<String>) {
        todo!()
    }

    fn update(self, modules: Vec<String>) {
        todo!()
    }
}
