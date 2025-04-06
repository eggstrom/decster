use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use itertools::Itertools;
use link::ModuleLink;
use serde::Deserialize;
use source::ModuleSource;

use crate::{config, out, source::ident::SourceIdent, state::State};

pub mod link;
pub mod source;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Module {
    #[serde(default)]
    import: HashSet<String>,
    #[serde(default)]
    pub user: Option<String>,

    #[serde(default)]
    files: BTreeMap<PathBuf, ModuleSource>,
    #[serde(default)]
    hard_links: BTreeMap<PathBuf, ModuleSource>,
    #[serde(default)]
    symlinks: BTreeMap<PathBuf, ModuleSource>,
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

    pub fn sources(&self) -> impl Iterator<Item = (&Path, &ModuleSource)> {
        self.files
            .iter()
            .chain(self.hard_links.iter())
            .chain(self.symlinks.iter())
            .map(|(path, source)| (path.as_path(), source))
            .unique()
            .sorted()
    }

    pub fn fetch_sources(&self, state: &mut State, module: &str) {
        let sources = self.sources();
        if sources.size_hint().0 > 0 {
            out!(1; "Fetching sources");
            for (path, source) in self.sources() {
                match source {
                    ModuleSource::Named(path) => {
                        let ident = SourceIdent::named(path.name.clone());
                        if let Some(source) = config::named_source(&path.name) {
                            if config::fetch() || !state.has_source(&ident, source) {
                                source.fetch_and_verify(state, &ident);
                            } else {
                                out!(2, Y; "{ident}"; "{source}");
                            }
                        } else if !config::has_source(&path.name) {
                            out!(2, R; "{}", path.name.magenta(); "Source isn't defined");
                        }
                    }
                    ModuleSource::Unnamed(source) => {
                        let ident = SourceIdent::unnamed(module, path);
                        if config::fetch() || !state.has_source(&ident, source) {
                            source.fetch_and_verify(state, &ident);
                        } else {
                            out!(2, Y; "{ident}"; "{source}");
                        }
                    }
                }
            }
        }
    }

    pub fn create_files(&self, state: &mut State, module: &str) {
        let files = self.files();
        if files.len() > 0 {
            out!(1; "Creating files");
            for link in files {
                link.create_files(state, module);
            }
        }
    }

    pub fn create_hard_links(&self, state: &mut State, module: &str) {
        let hard_links = self.hard_links();
        if hard_links.len() > 0 {
            out!(1; "Creating hard links");
            for link in hard_links {
                link.create_hard_links(state, module);
            }
        }
    }

    pub fn create_symlinks(&self, state: &mut State, module: &str) {
        let symlinks = self.symlinks();
        if symlinks.len() > 0 {
            out!(1; "Creating symlinks");
            for link in symlinks {
                link.create_symlinks(state, module);
            }
        }
    }
}
