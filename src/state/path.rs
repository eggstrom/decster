use std::{
    fmt::{self, Display, Formatter},
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use anyhow::Result;
use bincode::{Decode, Encode};
use crossterm::style::Stylize;

use crate::{
    env::Env,
    utils::{pretty::Pretty, sha256::Sha256Hash},
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
                    metadata.size() == *size
                        && Sha256Hash::from_file(path).is_ok_and(|h| h == *hash)
                }
                PathInfo::HardLink { size, hash } => {
                    metadata.size() == *size
                        && Sha256Hash::from_file(path).is_ok_and(|h| h == *hash)
                }
                PathInfo::Symlink { original } => path.read_link().is_ok_and(|o| o == *original),
            } {
                PathState::Matches
            } else {
                PathState::Differs
            }
        } else {
            PathState::Missing
        }
    }

    pub fn remove_if_owned<P>(&self, env: &Env, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        match self.state(path) {
            PathState::Matches => {
                if let PathKind::Directory = self.kind() {
                    let _ = fs::remove_dir(path);
                } else {
                    fs::remove_file(path)?;
                }
            }
            PathState::Differs => {
                eprintln!(
                    "{} Skipped {} (File has changed)",
                    "info:".yellow(),
                    env.tildefy(path).display()
                );
            }
            PathState::Missing => {
                eprintln!(
                    "{} Skipped {} (File is missing)",
                    "info:".yellow(),
                    env.tildefy(path).pretty()
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
            PathKind::Directory => "Directory".blue(),
            PathKind::File => "File".blue(),
            PathKind::HardLink => "Hard link".blue(),
            PathKind::Symlink => "Symlink".blue(),
        }
        .fmt(f)
    }
}

#[derive(PartialEq)]
pub enum PathState {
    Matches,
    Differs,
    Missing,
}
