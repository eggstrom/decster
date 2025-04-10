use std::{
    fmt::{self, Display, Formatter},
    fs, io,
    os::unix,
    path::{Path, PathBuf},
};

use bincode::{Decode, Encode};
use ident::SourceIdent;
use serde::Deserialize;

use crate::utils;

pub mod ident;
pub mod info;
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
    pub fn fetch(&self, ident: &SourceIdent) -> io::Result<()> {
        let source_path = ident.path();
        if source_path.exists() || source_path.is_symlink() {
            utils::fs::remove_all(&source_path)?;
        }

        match self {
            Source::Text(text) => self.fetch_text(&source_path, text),
            Source::Symlink(path) => self.fetch_symlink(&source_path, path),
            Source::Path(path) => self.fetch_path(&source_path, path),
        }
    }

    fn fetch_text(&self, source_path: &Path, text: &str) -> io::Result<()> {
        fs::write(source_path, text)
    }

    fn fetch_symlink(&self, source_path: &Path, path: &Path) -> io::Result<()> {
        unix::fs::symlink(path, source_path)
    }

    fn fetch_path(&self, source_path: &Path, path: &Path) -> io::Result<()> {
        utils::fs::copy_all(path, source_path)
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Source::Text(_) => "Text",
            Source::Symlink(_) => "Symlink",
            Source::Path(_) => "Path",
        }
        .fmt(f)
    }
}
