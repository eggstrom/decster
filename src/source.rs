use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use derive_more::From;
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize)]
pub enum Source {
    Path(PathBuf),
}

#[derive(Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct SourceName(String);

#[derive(Debug, Error, PartialEq)]
#[error("failed to parse source name")]
pub enum ParseSourceNameError {
    #[error("source names can't contain slashes")]
    ContainsSlash,
}

impl FromStr for SourceName {
    type Err = ParseSourceNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        (!s.contains('/'))
            .then_some(SourceName(s.to_string()))
            .ok_or(ParseSourceNameError::ContainsSlash)
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct SourcePath {
    source: SourceName,
    path: Option<PathBuf>,
}

impl SourcePath {
    pub fn source(&self) -> &str {
        &self.source.0
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
}

#[derive(Debug, Error, From, PartialEq)]
#[error("failed to parse source path")]
pub enum ParseSourcePathError {
    InvalidSourceName(ParseSourceNameError),
}

impl FromStr for SourcePath {
    type Err = ParseSourcePathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (source, path) = match s.split_once('/') {
            None => (s.parse()?, None),
            Some((source, path)) => (source.parse()?, Some(PathBuf::from(path))),
        };
        Ok(SourcePath { source, path })
    }
}
