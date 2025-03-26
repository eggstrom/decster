use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use anyhow::Result;
use crossterm::style::Stylize;
use file::ModuleFile;
use serde::Deserialize;

use crate::{
    config::Config,
    source::{name::SourceName, path::SourcePath},
    state::State,
};

pub mod file;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Module {
    #[serde(default)]
    import: HashSet<String>,

    #[serde(default)]
    files: HashMap<PathBuf, SourcePath>,
    #[serde(default)]
    hard_links: HashMap<PathBuf, SourcePath>,
    #[serde(default)]
    symlinks: HashMap<PathBuf, SourcePath>,
}

impl Module {
    fn files(&self) -> impl Iterator<Item = ModuleFile> {
        self.files
            .iter()
            .map(|(path, source)| ModuleFile::new(path.as_path(), source))
    }

    fn hard_links(&self) -> impl Iterator<Item = ModuleFile> {
        self.hard_links
            .iter()
            .map(|(path, source)| ModuleFile::new(path.as_path(), source))
    }

    fn symlinks(&self) -> impl Iterator<Item = ModuleFile> {
        self.symlinks
            .iter()
            .map(|(path, source)| ModuleFile::new(path.as_path(), source))
    }

    fn sources(&self) -> impl Iterator<Item = &SourceName> {
        self.files
            .values()
            .chain(self.hard_links.values())
            .chain(self.symlinks.values())
            .map(|source| &source.name)
    }

    pub fn add_sources(&self, config: &Config, state: &mut State) -> Result<()> {
        println!("  Adding sources");
        for name in self.sources() {
            let source = config.source(name)?;
            match state.add_source(name, source) {
                Ok(_) => println!("    {} {name} ({source})", "Added:".green()),
                Err(error) => println!("    {} {name} ({error})", "Failed:".red()),
            }
        }
        Ok(())
    }

    pub fn create_files(&self, state: &mut State, name: &str) -> Result<()> {
        println!("  Creating files");
        for files in self.files() {
            files.create_files(state, name);
        }
        println!("  Creating hard links");
        for hard_link in self.hard_links() {
            hard_link.create_hard_links(state, name);
        }
        println!("  Creating symlinks");
        for symlink in self.symlinks() {
            symlink.create_symlinks(state, name);
        }
        Ok(())
    }
}

pub enum ModuleFilter {
    All,
    Enabled,
    Disabled,
}
