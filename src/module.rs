use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use anyhow::Result;
use log::error;
use serde::Deserialize;

use crate::{
    config::Config,
    link::{Link, LinkMethod},
    source::SourcePath,
    state::State,
    utils,
};

#[derive(Debug, Deserialize)]
pub struct Module {
    #[serde(default)]
    import: HashSet<String>,
    link_method: Option<LinkMethod>,
    #[serde(default)]
    links: HashMap<PathBuf, SourcePath>,
}

impl Module {
    pub fn links<'a>(
        &'a self,
        name: &'a str,
        default_method: LinkMethod,
    ) -> impl Iterator<Item = Link<'a>> {
        self.links.iter().map(move |(path, source)| {
            Link::new(
                name,
                path,
                source,
                self.link_method.unwrap_or(default_method),
            )
        })
    }

    pub fn unwritable_paths(&self, state: &State) -> Vec<&Path> {
        let mut paths = Vec::new();
        for path in self.links.keys() {
            if !state.is_writable(path) {
                paths.push(path.as_path())
            }
        }
        paths
    }

    pub fn add_sources(&self, config: &Config, state: &mut State) -> Result<()> {
        for source in self.links.values() {
            let name = &source.name;
            let source = config.source(name)?;
            state.add_source(name, source)?;
        }
        Ok(())
    }

    pub fn enable(&self, state: &mut State, name: &str, default_method: LinkMethod) {
        for link in self.links(name, default_method) {
            if let Err(error) = link.enable(state) {
                error!("Couldn't enable link ({error:?})")
            }
        }
    }
}

pub enum ModuleFilter {
    All,
    Enabled,
    Disabled,
}
