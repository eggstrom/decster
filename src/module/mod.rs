use std::{
    collections::{BTreeMap, HashSet},
    path::PathBuf,
};

use crossterm::style::Stylize;
use link::ModuleLink;
use serde::Deserialize;

use crate::{
    global::config,
    out,
    source::{name::SourceName, path::SourcePath},
    state::State,
};

pub mod link;

#[derive(Debug, Deserialize)]
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
    }

    pub fn add_sources(&self, state: &mut State) {
        let sources = self.sources();
        if sources.size_hint().0 > 0 {
            out!(1, "", "Adding sources");
            for name in self.sources() {
                if let Some(source) = config::source(name) {
                    match state.add_source(name, source) {
                        Ok(_) => out!(2, added, "{name} ({source})"),
                        Err(err) => out!(2, failed, "{name} ({err})"),
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
