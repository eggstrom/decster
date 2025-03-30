use std::{
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use bincode::{Decode, Encode};

use crate::{
    out,
    utils::{self, fs::Sha256Hash, output::PathExt},
};

#[derive(Decode, Encode)]
pub enum PathInfo {
    Directory,
    File { size: u64, hash: Sha256Hash },
    HardLink { size: u64, hash: Sha256Hash },
    Symlink { original: PathBuf },
}

impl PathInfo {
    pub fn kind(&self) -> PathKind {
        match self {
            PathInfo::Directory => PathKind::Directory,
            PathInfo::File { .. } => PathKind::File,
            PathInfo::HardLink { .. } => PathKind::HardLink,
            PathInfo::Symlink { .. } => PathKind::Symlink,
        }
    }

    pub fn state<P>(&self, path: P) -> PathState
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        if let Ok(metadata) = path.symlink_metadata() {
            if match self {
                PathInfo::Directory => path.is_dir(),
                PathInfo::File { size, hash } => {
                    metadata.size() == *size && utils::fs::hash_file(path).is_ok_and(|h| h == *hash)
                }
                PathInfo::HardLink { size, hash } => {
                    metadata.size() == *size && utils::fs::hash_file(path).is_ok_and(|h| h == *hash)
                }
                PathInfo::Symlink { original } => path.read_link().is_ok_and(|o| o == *original),
            } {
                PathState::Owned
            } else {
                PathState::Changed
            }
        } else {
            PathState::Missing
        }
    }

    /// Removes `path` if it's contents match `self`. The returned `bool` tells
    /// if `path` was removed.
    pub fn remove_if_owned<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let kind = self.kind();
        match self.state(path) {
            PathState::Owned => {
                if let Err(err) = match kind {
                    PathKind::Directory => fs::remove_dir(path),
                    _ => fs::remove_file(path),
                } {
                    out!(2, R; "{}", path.display_kind(kind); "{err}");
                    return false;
                } else {
                    out!(2, G; "{}", path.display_kind(kind));
                }
            }
            PathState::Changed => {
                out!(2, Y; "{}", path.display_kind(kind); "File changed")
            }
            PathState::Missing => {
                out!(2, Y; "{}", path.display_kind(kind); "File missing")
            }
        }
        true
    }
}

pub enum PathKind {
    Directory,
    File,
    HardLink,
    Symlink,
}

pub enum PathState {
    Owned,
    Changed,
    Missing,
}
