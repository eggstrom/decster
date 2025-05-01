use std::{
    fs,
    os::unix,
    path::{Path, PathBuf},
};
#[cfg(feature = "http")]
use std::{fs::File, io};

use anyhow::Result;
use bincode::{Decode, Encode};
#[cfg(feature = "http")]
use reqwest::IntoUrl;
use serde::{Deserialize, Serialize};

use crate::global::env;
#[cfg(feature = "http")]
use crate::http;

pub mod hashable;
pub mod ident;
pub mod name;
pub mod path;

#[derive(
    Clone, Debug, Decode, Deserialize, Encode, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum Source {
    Text(String),
    Symlink(PathBuf),
    Path(PathBuf),
    #[cfg(feature = "http")]
    Url(String),
}

impl Source {
    pub fn fetch(&self, source_path: &Path) -> Result<()> {
        if source_path.exists() || source_path.is_symlink() {
            crate::fs::remove_all(source_path)?;
        }

        match self {
            Source::Text(text) => Self::fetch_text(source_path, text),
            Source::Symlink(path) => Self::fetch_symlink(source_path, path),
            Source::Path(path) => Self::fetch_path(source_path, &env::untildefy(path)),
            #[cfg(feature = "http")]
            Source::Url(url) => Self::fetch_url(source_path, url),
        }
    }

    fn fetch_text(source_path: &Path, text: &str) -> Result<()> {
        Ok(fs::write(source_path, text)?)
    }

    fn fetch_symlink(source_path: &Path, path: &Path) -> Result<()> {
        Ok(unix::fs::symlink(path, source_path)?)
    }

    fn fetch_path(source_path: &Path, path: &Path) -> Result<()> {
        crate::fs::copy_all(path, source_path)
    }

    #[cfg(feature = "http")]
    fn fetch_url<U>(source_path: &Path, url: U) -> Result<()>
    where
        U: IntoUrl,
    {
        let mut response = http::get(url)?;
        let mut file = File::create_new(source_path)?;
        io::copy(&mut response, &mut file)?;
        Ok(())
    }
}
