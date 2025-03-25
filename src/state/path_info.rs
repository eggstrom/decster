use std::{
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::utils::{self, Sha256Hash};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PathInfo {
    Directory,
    File { size: u64, hash: Sha256Hash },
    Link { path: PathBuf },
}

impl PathInfo {
    pub fn new_link<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        Ok(PathInfo::Link {
            path: path
                .read_link()
                .with_context(|| format!("Couldn't read symlink: {}", path.display()))?,
        })
    }

    pub fn new_file<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        Ok(PathInfo::File {
            size: path
                .metadata()
                .with_context(|| format!("Couldn't read file metadata: {}", path.display()))?
                .size(),
            hash: utils::hash_file(path)?,
        })
    }

    /// Checks whether the contents of `path` match `self`.
    pub fn matches<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        match self {
            PathInfo::Directory => path.is_dir(),
            PathInfo::File { size, hash } => {
                path.metadata()
                    .is_ok_and(|metadata| metadata.size() == *size)
                    && utils::hash_file(path).is_ok_and(|hash2| hash2 == *hash)
            }
            PathInfo::Link { path } => path.read_link().is_ok_and(|path2| path2 == *path),
        }
    }

    /// Removes `path` if it's contents match `self`.
    pub fn remove_if_matches<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        if self.matches(path) {
            if path.is_dir() {
                fs::remove_dir(path)?;
            } else {
                fs::remove_file(path)?;
            }
        }
        Ok(())
    }
}
