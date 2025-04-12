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
    source::{hashable::HashableSource, ident::SourceIdent, path::SourcePath},
    state::State,
    utils::sha256::Sha256Hash,
};

#[derive(Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum ModuleSource {
    Named(SourcePath),
    Unnamed(HashableSource),
}

impl ModuleSource {
    pub fn fetch(&self, state: &mut State, module: &str, path: &Path) -> Result<PathBuf> {
        let (path, info) = match self {
            ModuleSource::Named(path) => {
                if let Some(source) = config::named_source(&path.name) {
                    let ident = SourceIdent::named(path.name.clone());
                    (path.named_path(), Some((ident, source)))
                } else if config::has_source(&path.name) {
                    (path.config_path(), None)
                } else {
                    bail!("Source isn't defined");
                }
            }
            ModuleSource::Unnamed(source) => {
                let mut hasher = Sha256::new();
                hasher.update(module);
                hasher.update(path.to_string_lossy().as_ref());
                let hash = Sha256Hash::from(hasher.finalize());
                let ident = SourceIdent::unnamed(module, path);
                let path = paths::unnamed_sources().join(hash.to_string());
                (path, Some((ident, source)))
            }
        };

        if let Some((ident, source)) = info {
            if !state.is_source_fetched(&ident, source) {
                source.fetch(&path)?;
                state.add_source(&ident, source);
            }
        }
        Ok(path)
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
