use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use crossterm::style::Stylize;
use itertools::Itertools;
use link::ModuleLink;
use serde::Deserialize;

use crate::{
    config, out,
    source::{name::SourceName, path::SourcePath},
    state::State,
};

pub mod link;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Module {
    #[serde(default)]
    import: HashSet<String>,

    #[serde(default)]
    files: BTreeMap<PathBuf, SourcePath>,
    #[serde(default)]
    hard_links: BTreeMap<PathBuf, SourcePath>,
    #[serde(default)]
    symlinks: BTreeMap<PathBuf, SourcePath>,
}

impl Module {
    pub fn parse<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        Ok(toml::from_str(&fs::read_to_string(path)?)?)
    }

    pub fn files(&self) -> impl ExactSizeIterator<Item = ModuleLink> {
        self.files
            .iter()
            .map(|(path, source)| ModuleLink::new(path.as_path(), source))
    }

    pub fn hard_links(&self) -> impl ExactSizeIterator<Item = ModuleLink> {
        self.hard_links
            .iter()
            .map(|(path, source)| ModuleLink::new(path.as_path(), source))
    }

    pub fn symlinks(&self) -> impl ExactSizeIterator<Item = ModuleLink> {
        self.symlinks
            .iter()
            .map(|(path, source)| ModuleLink::new(path.as_path(), source))
    }

    pub fn sources(&self) -> impl Iterator<Item = &SourceName> {
        self.files
            .values()
            .chain(self.hard_links.values())
            .chain(self.symlinks.values())
            .map(|source| &source.name)
            .unique()
            .sorted()
    }

    pub fn fetch_sources(&self, state: &mut State) {
        let sources = self.sources();
        if sources.size_hint().0 > 0 {
            out!(1, "", "Fetching sources");
            for name in self.sources() {
                if let Some(source) = config::source(name) {
                    if !config::fetch() && state.has_source(name, source) {
                        out!(2, skipped, "{name} (Already fetched)");
                    } else {
                        match state.fetch_source(name, source) {
                            Ok(_) => out!(2, fetched, "{name} ({source})"),
                            Err(err) => out!(2, failed, "{name} ({err})"),
                        }
                    }
                } else {
                    out!(2, failed, "{} (Source isn't defined)", name.magenta());
                }
            }
        }
    }

    pub fn create_files(&self, state: &mut State, module: &str) {
        let files = self.files();
        if files.len() > 0 {
            out!(1, "", "Creating files");
            for file in files {
                file.create_files(state, module);
            }
        }
    }

    pub fn create_hard_links(&self, state: &mut State, module: &str) {
        let hard_links = self.hard_links();
        if hard_links.len() > 0 {
            out!(1, "", "Creating hard links");
            for hard_link in hard_links {
                hard_link.create_hard_links(state, module);
            }
        }
    }

    pub fn create_symlinks(&self, state: &mut State, module: &str) {
        let symlinks = self.symlinks();
        if symlinks.len() > 0 {
            out!(1, "", "Creating symlinks");
            for symlink in symlinks {
                symlink.create_symlinks(state, module);
            }
        }
    }
}
