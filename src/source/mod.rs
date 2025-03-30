use std::{
    fmt::{self, Display, Formatter},
    fs, io,
    path::{Path, PathBuf},
};

use bincode::{Decode, Encode};
use name::SourceName;
use serde::Deserialize;

use crate::utils;

pub mod name;
pub mod path;

#[derive(Clone, Debug, Decode, Deserialize, Encode, Eq, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum Source {
    #[serde(skip)]
    Static,
    Text(String),
    Path(PathBuf),
}

impl Source {
    pub fn fetch(&self, name: &SourceName) -> io::Result<()> {
        if let Source::Static = self {
            return Ok(());
        }

        let source_path = name.path();
        if source_path.exists() {
            utils::fs::remove_all(&source_path)?;
        }

        match self {
            Source::Static => unreachable!(),
            Source::Text(text) => self.fetch_text(&source_path, text),
            Source::Path(path) => self.fetch_path(&source_path, path),
        }
    }

    fn fetch_text(&self, source_path: &Path, text: &str) -> io::Result<()> {
        fs::write(source_path, text)
    }

    fn fetch_path(&self, source_path: &Path, path: &Path) -> io::Result<()> {
        utils::fs::copy_all(path, source_path)
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Source::Static => "Static",
            Source::Text(_) => "Text",
            Source::Path(_) => "Path",
        }
        .fmt(f)
    }
}
