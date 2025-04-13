use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
    str::FromStr,
};

use anyhow::Result;
use derive_more::From;
use serde::{
    Deserialize, Deserializer,
    de::{self, Visitor},
};
use thiserror::Error;

use crate::utils::pretty::Pretty;

use super::name::{ParseSourceNameError, SourceName};

#[derive(Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct SourcePath {
    pub name: SourceName,
    pub path: Option<PathBuf>,
}

impl SourcePath {
    pub fn named_path(&self) -> PathBuf {
        let mut full_path = self.name.named_path();
        if let Some(path) = &self.path {
            full_path.push(path);
        }
        full_path
    }

    pub fn config_path(&self) -> PathBuf {
        let mut full_path = self.name.config_path();
        if let Some(path) = &self.path {
            full_path.push(path);
        }
        full_path
    }
}

impl Display for SourcePath {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self.path {
            Some(path) => write!(f, "{}/{}", self.name, path.pretty()),
            None => self.name.fmt(f),
        }
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
        Ok(SourcePath { name: source, path })
    }
}

struct SourcePathVisitor;

impl Visitor<'_> for SourcePathVisitor {
    type Value = SourcePath;

    fn expecting(&self, f: &mut Formatter) -> fmt::Result {
        "a source name followed by an optional path".fmt(f)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        v.parse().map_err(de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for SourcePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(SourcePathVisitor)
    }
}
