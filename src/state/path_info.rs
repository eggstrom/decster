use std::{
    fs, io,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use bincode::{Decode, Encode};
use crossterm::style::Stylize;

use crate::{
    global::config,
    out,
    utils::{self, fs::Sha256Hash, output::Pretty},
};

#[derive(Decode, Encode)]
pub enum PathInfo {
    Directory,
    File { size: u64, hash: Sha256Hash },
    HardLink { size: u64, hash: Sha256Hash },
    Symlink { path: PathBuf },
}

impl PathInfo {
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

    fn is_hard_link_and<F>(&self, f: F) -> bool
    where
        F: FnOnce(u64, &Sha256Hash) -> bool,
    {
        match self {
            PathInfo::HardLink { size, hash } => f(*size, hash),
            _ => false,
        }
    }

    fn is_symlink_and<F>(&self, f: F) -> bool
    where
        F: FnOnce(&Path) -> bool,
    {
        match self {
            PathInfo::Symlink { path } => f(&path),
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
            } else if self.is_hard_link_and(|size, hash| {
                metadata.size() == size && utils::fs::hash_file(path).is_ok_and(|h| h == *hash)
            }) {
                PathState::OwnedHardLink
            } else if self
                .is_symlink_and(|link_path| path.read_link().is_ok_and(|p| p == link_path))
            {
                PathState::OwnedSymlink
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
                fs::remove_dir(path)?;
                out!("{} {}", "    Removed:".green(), path.pretty());
            }
            PathState::OwnedFile | PathState::OwnedHardLink | PathState::OwnedSymlink => {
                fs::remove_file(path)?;
                out!("{} {}", "    Removed:".green(), path.pretty());
            }
            PathState::Changed => {
                out!(
                    "{} {} (File changed)",
                    "  Skipping:".yellow(),
                    path.pretty()
                )
            }
            PathState::Missing => {
                out!(
                    "{} {} (File missing)",
                    "  Skipping:".yellow(),
                    path.pretty()
                )
            }
        }
        Ok(())
    }
}

pub enum PathState {
    OwnedDirectory,
    OwnedFile,
    OwnedHardLink,
    OwnedSymlink,
    Changed,
    Missing,
}
