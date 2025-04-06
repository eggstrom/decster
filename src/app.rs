use std::{
    collections::{BTreeSet, HashSet},
    fs,
    path::Path,
};

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
        self.hash_dir("Config", paths::config_sources(), &sources)?;
        self.hash_dir("Named", paths::named_sources(), &sources)?;
        Ok(())
    }

    fn hash_dir(&self, text: &str, dir: &Path, sources: &HashSet<SourceName>) -> Result<()> {
        let mut sources: Vec<_> = fs::read_dir(dir)?
            .filter_map(Result::ok)
            .map(|entry| (SourceName::from(entry.file_name()), entry.path()))
            .filter(|(name, _)| sources.is_empty() || sources.contains(name))
            .collect();
        sources.sort_unstable();
        if !sources.is_empty() {
            out!(0; "{} sources", text);
        }
        for (name, path) in sources {
            out!(1; "{}: {}", name, path.hash_all()?)
        }
        Ok(())
    }
}
