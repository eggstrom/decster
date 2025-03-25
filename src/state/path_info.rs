use std::{
    fs, io,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use crossterm::style::Stylize;
use serde::{Deserialize, Serialize};

use crate::utils::{self, fs::Sha256Hash, output::Pretty};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PathInfo {
    Directory,
    File { size: u64, hash: Sha256Hash },
    Link { path: PathBuf },
}

impl PathInfo {
    pub fn new_dir() -> Self {
        PathInfo::Directory
    }

    pub fn new_file<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        Ok(PathInfo::File {
            size: path.symlink_metadata()?.size(),
            hash: utils::fs::hash_file(path)?,
        })
    }

    pub fn new_link<P>(path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        PathInfo::Link { path: path.into() }
    }

    fn is_dir_and<F>(&self, f: F) -> bool
    where
        F: FnOnce() -> bool,
    {
        match self {
            PathInfo::Directory => f(),
            _ => false,
        }
    }

    fn is_file_and<F>(&self, f: F) -> bool
    where
        F: FnOnce(u64, &Sha256Hash) -> bool,
    {
        match self {
            PathInfo::File { size, hash } => f(*size, hash),
            _ => false,
        }
    }

    fn is_link_and<F>(&self, f: F) -> bool
    where
        F: FnOnce(&Path) -> bool,
    {
        match self {
            PathInfo::Link { path } => f(&path),
            _ => false,
        }
    }

    pub fn state<P>(&self, path: P) -> PathState
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        if let Ok(metadata) = path.symlink_metadata() {
            if self.is_dir_and(|| path.is_dir()) {
                PathState::OwnedDirectory
            } else if self.is_file_and(|size, hash| {
                metadata.size() == size && utils::fs::hash_file(path).is_ok_and(|h| h == *hash)
            }) {
                PathState::OwnedFile
            } else if self.is_link_and(|link_path| path.read_link().is_ok_and(|p| p == link_path)) {
                PathState::OwnedSoftLink
            } else {
                PathState::Changed
            }
        } else {
            PathState::Missing
        }
    }

    /// Removes `path` if it's contents match `self`.
    pub fn remove_if_owned<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        match self.state(path) {
            PathState::OwnedDirectory => {
                let _ = fs::remove_dir(path);
            }
            PathState::OwnedFile | PathState::OwnedSoftLink => {
                fs::remove_file(path)?;
                println!("{} {}", "  Removed:".green(), path.pretty());
            }
            PathState::Changed => println!("{} {}", "  Changed:".yellow(), path.pretty()),
            PathState::Missing => println!("{} {}", "  Missing:".yellow(), path.pretty()),
        }
        Ok(())
    }
}

pub enum PathState {
    OwnedDirectory,
    OwnedFile,
    OwnedSoftLink,
    Changed,
    Missing,
}
