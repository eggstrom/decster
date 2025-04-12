use std::{
    fmt::{self, Display, Formatter},
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use anyhow::Result;
use bincode::{Decode, Encode};
use crossterm::style::Stylize;

use crate::utils::{
    output::PrettyPathExt,
    sha256::{PathHash, Sha256Hash},
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
                    metadata.size() == *size && path.hash_file().is_ok_and(|h| h == *hash)
                }
                PathInfo::HardLink { size, hash } => {
                    metadata.size() == *size && path.hash_file().is_ok_and(|h| h == *hash)
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

    pub fn remove_if_owned<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        match self.state(path) {
            PathState::Owned => match self.kind() {
                PathKind::Directory => fs::remove_dir(path)?,
                _ => fs::remove_file(path)?,
            },
            PathState::Changed => {
                eprintln!(
                    "{} Skipped {} (File has changed)",
                    "info:".yellow(),
                    path.pretty()
                );
            }
            PathState::Missing => {
                eprintln!(
                    "{} Skipped {} (File is missing)",
                    "info:".yellow(),
                    path.pretty()
                );
            }
        }
        Ok(())
    }
}

pub enum PathKind {
    Directory,
    File,
    HardLink,
    Symlink,
}

impl Display for PathKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            PathKind::Directory => "Directory".grey(),
            PathKind::File => "File".green(),
            PathKind::HardLink => "HardLink".cyan(),
            PathKind::Symlink => "Symlink".blue(),
        }
        .fmt(f)
    }
}

pub enum PathState {
    Owned,
    Changed,
    Missing,
}
