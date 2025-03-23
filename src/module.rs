use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use anyhow::Result;
use log::{error, info, warn};
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
    pub fn links(&self, default_method: LinkMethod) -> impl Iterator<Item = Link> {
        self.links.iter().map(move |(path, source)| {
            Link::new(path, source, self.link_method.unwrap_or(default_method))
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

    pub fn enable(&self, default_method: LinkMethod, state: &mut State) {
        for link in self.links(default_method) {
            if let Err(error) = utils::remove_all(link.path) {
                error!("{error}");
            }

            match link.enable() {
                Ok(()) => {
                    if let Err(error) = state.add_file(link.path, link.method) {
                        error!("{error:?}");
                    }
                }
                Err(error) => error!("{error:?}"),
            }
        }
    }

    pub fn disable(&self, state: &mut State) {
        for path in self.links.keys() {
            if !path.exists() {
                warn!("File doesn't exist: {}", path.display());
                continue;
            }

            if state.is_writable(path) {
                match utils::remove_all(path) {
                    Ok(()) => {
                        info!("Removed file: {}", path.display());
                        state.remove_file(path)
                    }
                    Err(error) => error!("{error:?}"),
                }
            }
        }
    }
}

pub enum ModuleFilter {
    All,
    Enabled,
    Disabled,
}
