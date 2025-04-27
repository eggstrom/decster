use std::{fmt::Display, path::Path};

use anyhow::{Result, bail};
use clap::Parser;
use crossterm::style::Stylize;

use crate::{
    cli::{Cli, Command},
    global::{self, config, env},
    globs::Globs,
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
            Command::Enable { modules } => app.enable(&modules)?,
            Command::Disable { modules } => app.disable(&modules)?,
            Command::Update { modules } => app.update(&modules)?,
            Command::List => app.list(),
            Command::Paths => app.paths()?,
            Command::Hash { sources } => app.hash(&sources)?,
        }
        Ok(())
    }

    fn enable(&mut self, modules: &[String]) -> Result<()> {
        let globs = Globs::strict(modules)?;
        let mut has_enabled = false;
        for (name, module) in config::modules_matching_globs(&globs) {
            let modules = module.import(name)?;
            if !self.state.is_module_enabled(name) {
                has_enabled = true;
                if let Err(err) = self.state.enable_module(name, modules) {
                    eprintln!("{} {err:?}", "error:".red());
                } else {
                    println!("Enabled {}", name.magenta());
                }
            }
        }
        if !has_enabled {
            bail!("{} didn't match any disabled modules", modules.pretty());
        }
        self.state.save()
    }

    fn disable(&mut self, modules: &[String]) -> Result<()> {
        let globs = Globs::strict(modules)?;
        let matches = self.state.modules_matching_globs(&globs);
        if matches.is_empty() {
            bail!("{} didn't match any enabled modules", modules.pretty());
        }
        for module in matches {
            if let Err(err) = self.state.disable_module(&module) {
                eprintln!("{} {err:?}", "error:".red());
            } else {
                println!("Disabled {}", module.magenta());
            }
        }
        self.state.save()
    }

    fn update(&mut self, modules: &[String]) -> Result<()> {
        if modules.is_empty() {
            match self.update_inner(self.state.modules()) {
                Ok(false) => bail!("There are no enabled modules"),
                Err(err) => eprintln!("{} {err}", "error:".red()),
                _ => (),
            }
        } else {
            let globs = Globs::permissive(modules)?;
            match self.update_inner(self.state.modules_matching_globs(&globs)) {
                Ok(false) => {
                    bail!("{} didn't match any enabled modules", modules.pretty());
                }
                Err(err) => eprintln!("{} {err}", "error:".red()),
                _ => (),
            }
        }
        self.state.save()
    }

    fn update_inner<I>(&mut self, modules: I) -> Result<bool>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let mut has_updated = false;
        for name in modules {
            has_updated = true;
            let name = name.as_ref();
            let modules = config::module(name).map(|(_, module)| module.import(name));
            self.state.update_module(name, modules.transpose()?)?;
            println!("Updated {}", name.magenta());
        }
        Ok(has_updated)
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

    fn hash(&self, sources: &[String]) -> Result<()> {
        if sources.is_empty() {
            if !Self::hash_inner(config::static_sources(), self.state.sources()) {
                bail!("There are no fetched sources");
            }
        } else {
            let globs = Globs::permissive(sources)?;
            if !Self::hash_inner(
                config::static_sources_matching_globs(&globs),
                self.state.sources_matching_globs(&globs),
            ) {
                bail!("{} didn't match any fetched sources", sources.pretty());
            }
        };
        Ok(())
    }

    fn hash_inner<'a, 'b, S, D>(static_sources: S, dynamic_sources: D) -> bool
    where
        S: Iterator<Item = &'a SourceName>,
        D: Iterator<Item = &'b SourceIdent>,
    {
        let mut has_sources = false;
        for source in static_sources {
            print!("({}) ", "Static".blue());
            Self::print_source_hash(source, &env::static_sources().join(source));
            has_sources = true;
        }
        for source in dynamic_sources {
            print!("({}) ", "Dynamic".blue());
            Self::print_source_hash(source, &source.path());
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
