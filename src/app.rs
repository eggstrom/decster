use std::{fmt::Display, path::Path};

use anyhow::{Result, bail};
use clap::Parser;
use crossterm::style::Stylize;

use crate::{
    cli::{Cli, Command},
    global::{self, config, env},
    module::set::ModuleSet,
    source::{ident::SourceIdent, name::SourceName},
    state::State,
    utils::{pretty::Pretty, sha256::Sha256Hash},
};

pub struct App {
    state: State,
}

impl App {
    pub fn run() -> Result<()> {
        let cli = Cli::parse();
        global::load(cli.config, cli.behavior)?;
        let state = State::load()?;
        let mut app = App { state };

        match cli.command {
            Command::Enable { modules } => app.enable(modules)?,
            Command::Disable { modules } => app.disable(modules.into_iter().collect())?,
            Command::Update { modules } => app.update(modules)?,
            Command::List => app.list(),
            Command::Paths => app.paths()?,
            Command::Hash { sources } => app.hash(sources)?,
        }
        Ok(())
    }

    fn enable(&mut self, modules: Vec<String>) -> Result<()> {
        let mut has_enabled = false;
        for name in config::modules_matching_globs(&modules)? {
            let modules = ModuleSet::new(name)?;
            if !self.state.is_module_enabled(name) {
                has_enabled = true;
                if let Err(err) = self.state.enable_module(name, &modules) {
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
        self.state.save()
    }

    fn disable(&mut self, modules: Vec<String>) -> Result<()> {
        self.state.disable_modules_matching_globs(&modules)?;
        self.state.save()
    }

    fn update(&mut self, modules: Vec<String>) -> Result<()> {
        if modules.is_empty() {
            self.state.update_all_modules()?;
        } else {
            self.state.update_modules_matching_globs(&modules)?;
        }
        self.state.save()
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
                println!("  {} ({})", env::tildefy(path).pretty(), info.kind());
            }
        }
        Ok(())
    }

    fn hash(&self, sources: Vec<String>) -> Result<()> {
        if sources.is_empty() {
            if !Self::hash_inner(config::static_sources(), self.state.sources()) {
                bail!("There are no fetched sources");
            }
        } else {
            if !Self::hash_inner(
                config::static_sources_matching_globs(&sources)?,
                self.state.sources_matching_globs(&sources)?,
            ) {
                let sources = sources.as_slice();
                bail!("{} didn't match any fetched sources", sources.pretty());
            }
        };
        Ok(())
    }

    fn hash_inner<'a, S, D>(static_sources: S, dynamic_sources: D) -> bool
    where
        S: Iterator<Item = &'static SourceName>,
        D: Iterator<Item = &'a SourceIdent>,
    {
        let mut has_sources = false;
        for source in static_sources {
            print!("({}) ", "Static".blue());
            Self::print_source_hash(source, &env::static_sources().join(source.to_string()));
            has_sources = true;
        }
        for source in dynamic_sources {
            print!("({}) ", "Dynamic".blue());
            Self::print_source_hash(source, &env::static_sources().join(source.to_string()));
            has_sources = true;
        }
        has_sources
    }

    fn print_source_hash<D>(ident: D, path: &Path)
    where
        D: Display,
    {
        print!("{ident}: ");
        match Sha256Hash::from_path(path) {
            Ok(hash) => println!("{hash}"),
            Err(err) => println!("{} {err:?}", "error:".red()),
        }
    }
}
