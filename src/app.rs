use std::collections::{BTreeSet, HashSet};

use anyhow::Result;
use clap::Parser;

use crate::{
    cli::{Cli, Command, InfoArgs},
    config, out, paths,
    source::name::SourceName,
    state::State,
    users::Users,
    utils::{output::PathDisplay, sha256::PathHash},
};

pub struct App {
    users: Users,
    state: State,
}

impl App {
    pub fn run() -> Result<()> {
        let cli = Cli::parse();
        paths::load(cli.config)?;
        config::load(cli.behavior)?;
        let app = App {
            users: Users::new(),
            state: State::load()?,
        };

        match cli.command {
            Command::Info(args) => app.info(args),
            Command::Enable { modules } => app.enable(modules.into_iter().collect())?,
            Command::Disable { modules } => app.disable(modules.into_iter().collect())?,
            Command::Update { modules } => app.update(modules.into_iter().collect())?,
            Command::Hash { sources } => app.hash(sources.into_iter().collect())?,
        }
        Ok(())
    }

    fn info(self, args: InfoArgs) {
        let (modules, filter) = args.modules();
        for (name, module, paths) in self.state.modules(modules, filter) {
            out!(0; "Module {}", name.magenta());
            let files = module.files();
            let hard_links = module.hard_links();
            let symlinks = module.symlinks();

            if files.len() > 0 {
                out!(1; "Files");
                files.for_each(|link| out!(2; "{link}"));
            }
            if hard_links.len() > 0 {
                out!(1; "Hard links");
                hard_links.for_each(|link| out!(2; "{link}"));
            }
            if symlinks.len() > 0 {
                out!(1; "Symlinks");
                symlinks.for_each(|link| out!(2; "{link}"));
            }

            if let Some(paths) = paths {
                out!(1; "Owned paths");
                for (path, info) in paths {
                    out!(2; "{}", path.display_kind(info.kind()))
                }
            }
        }
    }

    fn enable(mut self, modules: BTreeSet<String>) -> Result<()> {
        if modules.is_empty() {
            self.state.enable_all_modules(&mut self.users);
        } else {
            for module in modules {
                self.state.enable_module(&mut self.users, &module);
            }
        }
        self.state.save()
    }

    fn disable(mut self, modules: BTreeSet<String>) -> Result<()> {
        if modules.is_empty() {
            self.state.disable_all_modules();
        } else {
            for module in modules {
                self.state.disable_module(&module);
            }
        }
        self.state.save()
    }

    fn update(mut self, modules: BTreeSet<String>) -> Result<()> {
        if modules.is_empty() {
            self.state.update_all_modules(&mut self.users);
        } else {
            for module in modules {
                self.state.update_module(&mut self.users, &module);
            }
        }
        self.state.save()
    }

    fn hash(self, sources: HashSet<SourceName>) -> Result<()> {
        if sources.is_empty() {
            let sources = self.state.sources();
            if sources.len() == 0 {
                out!(0, R; "There are no fetched sources");
                return Ok(());
            }
            for (name, source) in self.state.sources() {
                out!(0; "{name}: {}", source.path(&name).hash_all()?)
            }
        } else {
            for name in sources {
                match self.state.source(&name) {
                    Some(source) => out!(0; "{name}: {}", source.path(&name).hash_all()?),
                    None => out!(0, R; "Source {name} isn't defined"),
                }
            }
        }
        Ok(())
    }
}
