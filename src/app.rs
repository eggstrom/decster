use std::{collections::HashSet, fs, path::Path};

use anyhow::{Result, bail};
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
            Command::Enable { modules } => app.enable(modules)?,
            Command::Disable { modules } => app.disable(modules.into_iter().collect())?,
            Command::Update { modules } => app.update(modules)?,
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

    fn enable(mut self, modules: Vec<String>) -> Result<()> {
        let mut has_enabled = false;
        for (name, module) in config::modules_matching_globs(&modules)? {
            if !self.state.has_module(name) {
                has_enabled = true;
                self.state.enable_module(&mut self.users, name, module);
            }
        }
        if !has_enabled {
            bail!("Patterns didn't match any disabled modules");
        }
        self.state.save()
    }

    fn disable(mut self, modules: Vec<String>) -> Result<()> {
        self.state.disable_modules_matching_globs(&modules)?;
        self.state.save()
    }

    fn update(mut self, modules: Vec<String>) -> Result<()> {
        if modules.is_empty() {
            self.state.update_all_modules(&mut self.users)?;
        } else {
            self.state
                .update_modules_matching_globs(&mut self.users, &modules)?;
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
