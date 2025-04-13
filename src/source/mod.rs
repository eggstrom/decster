use std::{
    fs,
    os::unix,
    path::{Path, PathBuf},
};

use anyhow::Result;
use bincode::{Decode, Encode};
use serde::Deserialize;

use crate::{env::Env, utils};

pub mod hashable;
pub mod ident;
pub mod name;
pub mod path;

#[derive(Clone, Debug, Decode, Deserialize, Encode, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum Source {
    Text(String),
    Symlink(PathBuf),
    Path(PathBuf),
}

impl Source {
    pub fn fetch(&self, env: &Env, source_path: &Path) -> Result<()> {
        if source_path.exists() || source_path.is_symlink() {
            utils::fs::remove_all(source_path)?;
        }

        match self {
            Source::Text(text) => self.fetch_text(source_path, text),
            Source::Symlink(path) => self.fetch_symlink(source_path, path),
            Source::Path(path) => self.fetch_path(source_path, &env.untildefy(path)),
        }
    }

    fn fetch_text(&self, source_path: &Path, text: &str) -> Result<()> {
        Ok(fs::write(source_path, text)?)
    }

    fn fetch_symlink(&self, source_path: &Path, path: &Path) -> Result<()> {
        Ok(unix::fs::symlink(path, source_path)?)
    }

    fn fetch_path(&self, source_path: &Path, path: &Path) -> Result<()> {
        utils::fs::copy_all(path, source_path)
    }
}
