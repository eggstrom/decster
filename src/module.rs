use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

use anyhow::{Result, bail};
use serde::Deserialize;

use crate::{
    link::{Link, LinkMethod},
    source::SourcePath,
    utils,
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

    pub fn is_enabled(&self, default_method: LinkMethod) -> Result<bool> {
        Ok(self
            .links(default_method)
            .all(|link| link.is_enabled().is_ok_and(|enabled| enabled)))
    }

    pub fn enable(&self, default_method: LinkMethod) -> Result<()> {
        let mut created_files = Vec::new();

        for link in self.links(default_method) {
            match link.enable() {
                Ok(()) => created_files.push(link.path.to_path_buf()),
                Err(error) => {
                    for path in created_files.iter() {
                        utils::remove_all(path)?;
                        utils::remove_dir_components(path);
                    }
                    bail!(error);
                }
            }
        }
        Ok(())
    }

    pub fn disable(&self, default_method: LinkMethod) -> Result<()> {
        for link in self.links(default_method) {
            link.disable()?;
        }
        Ok(())
    }
}

pub enum ModuleFilter {
    All,
    Enabled,
    Disabled,
}
