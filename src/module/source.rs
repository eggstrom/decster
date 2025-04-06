use std::{
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};
use crossterm::style::Stylize;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{
    config, paths,
    source::{info::SourceInfo, path::SourcePath},
    utils::sha256::Sha256Hash,
};

#[derive(Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum ModuleSource {
    Named(SourcePath),
    Unnamed(SourceInfo),
}

impl ModuleSource {
    pub fn path(&self, module: &str, path: &Path) -> Result<PathBuf> {
        Ok(match self {
            ModuleSource::Named(path) if config::has_named_source(&path.name) => path.named_path(),
            ModuleSource::Named(path) if config::has_source(&path.name) => path.config_path(),
            ModuleSource::Named(path) => bail!("Source {} isn't defined", path.name),
            ModuleSource::Unnamed(_) => {
                let mut hasher = Sha256::new();
                hasher.update(module);
                hasher.update(path.to_string_lossy().as_ref());
                let hash = Sha256Hash::from(hasher.finalize());
                paths::unnamed_sources().join(hash.to_string())
            }
        })
    }
}

impl Display for ModuleSource {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ModuleSource::Named(path) => path.fmt(f),
            ModuleSource::Unnamed(_) => "Unnamed".grey().fmt(f),
        }
    }
}
