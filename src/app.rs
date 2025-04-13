use std::{collections::HashSet, fs, path::Path};

use anyhow::{Result, bail};
use clap::Parser;
use crossterm::style::Stylize;

use crate::{
    cli::{Cli, Command},
    config,
    env::Env,
    module::set::ModuleSet,
    source::name::SourceName,
    state::State,
    utils::{pretty::Pretty, sha256::PathHash},
};

pub struct App {
    env: Env,
    state: State,
}

impl App {
    pub fn run() -> Result<()> {
        let cli = Cli::parse();
        let env = Env::load(cli.config)?;
        config::load(&env, cli.behavior)?;
        let state = State::load(&env)?;
        let mut app = App { env, state };

        match cli.command {
            Command::Enable { modules } => app.enable(modules)?,
            Command::Disable { modules } => app.disable(modules.into_iter().collect())?,
            Command::Update { modules } => app.update(modules)?,
            Command::List => app.list(),
            Command::Paths => app.paths()?,
            Command::Hash { sources } => app.hash(sources.into_iter().collect())?,
        }
        Ok(())
    }

    fn enable(&mut self, modules: Vec<String>) -> Result<()> {
        let mut has_enabled = false;
        for name in config::modules_matching_globs(&modules)? {
            let modules = ModuleSet::new(name)?;
            if !self.state.is_module_enabled(name) {
                has_enabled = true;
                if let Err(err) = self.state.enable_module(&mut self.env, name, &modules) {
                    eprintln!("{} {err:?}", "error:".red());
                } else {
                    println!("Enabled {}", name.magenta());
                }
            }
        }
        if !has_enabled {
            let modules = modules.as_slice();
            bail!("{} didn't match any disabled modules", modules.pretty());
        }
        self.state.save(&self.env)
    }

    fn disable(&mut self, modules: Vec<String>) -> Result<()> {
        self.state
            .disable_modules_matching_globs(&self.env, &modules)?;
        self.state.save(&self.env)
    }

    fn update(&mut self, modules: Vec<String>) -> Result<()> {
        if modules.is_empty() {
            self.state.update_all_modules(&mut self.env)?;
        } else {
            self.state
                .update_modules_matching_globs(&mut self.env, &modules)?;
        }
        self.state.save(&self.env)
    }

    fn list(&self) {
        for (name, _) in config::modules() {
            let enabled = self.state.is_module_enabled(name);
            let state = match enabled {
                true => "Enabled".green(),
                false => "Disabled".red(),
            };
            println!("{} ({state})", name.magenta());
        }
    }

    fn paths(&self) -> Result<()> {
        let owned_paths = self.state.owned_paths();
        if owned_paths.len() == 0 {
            println!("There are no owned paths");
        }
        for (module, paths) in self.state.owned_paths() {
            println!("Paths owned by module {}:", module.magenta());
            for (path, info) in paths {
                println!("  {} ({})", self.env.tildefy(path).pretty(), info.kind());
            }
        }
        Ok(())
    }

    fn hash(&self, sources: HashSet<SourceName>) -> Result<()> {
        self.hash_dir("Config", self.env.config_sources(), &sources)?;
        self.hash_dir("Named", self.env.named_sources(), &sources)?;
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
            println!("{} sources:", text);
            for (name, path) in sources {
                println!("  {}: {}", name, path.hash_all()?)
            }
        }
        Ok(())
    }
}
