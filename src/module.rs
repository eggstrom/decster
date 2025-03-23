use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};
use serde::Deserialize;

use crate::{
    link::{Link, LinkMethod},
    source::SourcePath,
    state::State,
};

#[derive(Debug, Deserialize)]
pub struct Module {
    #[serde(default)]
    pub import: HashSet<String>,
    pub link_method: Option<LinkMethod>,
    #[serde(default)]
    pub links: HashMap<PathBuf, SourcePath>,
}

impl Module {
    pub fn links(&self, default_method: LinkMethod) -> impl Iterator<Item = Link> {
        self.links.iter().map(move |(path, source)| {
            Link::new(path, source, self.link_method.unwrap_or(default_method))
        })
    }

    pub fn unwrittable_paths(&self, default_method: LinkMethod, state: &mut State) -> Vec<&Path> {
        let mut unwrittable_paths = Vec::new();
        for link in self.links(default_method) {
            if state.check(link.path) {
                unwrittable_paths.push(link.path)
            }
        }
        unwrittable_paths
    }

    pub fn is_enabled(&self, default_method: LinkMethod) -> Result<bool> {
        Ok(self
            .links(default_method)
            .all(|link| link.is_enabled().is_ok_and(|enabled| enabled)))
    }

    pub fn enable(&self, default_method: LinkMethod, state: &mut State) -> Result<()> {
        for link in self.links(default_method) {
            if state.check(link.path) {
                match link.enable() {
                    Ok(()) => state.add_file(link.path, link.method)?,
                    Err(error) => {
                        self.disable(default_method, state)?;
                        bail!(error);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn disable(&self, default_method: LinkMethod, state: &mut State) -> Result<()> {
        for link in self.links(default_method) {
            if state.check(link.path) {
                if let Ok(()) = link.disable() {
                    state.remove_file(link.path);
                }
            }
        }
        Ok(())
    }
}

pub enum ModuleFilter {
    All,
    Enabled,
    Disabled,
}
