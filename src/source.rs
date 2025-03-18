use std::{
    borrow::Borrow,
    fmt::{self, Display, Formatter},
    path::PathBuf,
    str::FromStr,
};

use crossterm::style::Stylize;
use derive_more::From;
use serde::{
    Deserialize, Deserializer,
    de::{self, Visitor},
};
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub enum Source {
    #[serde(rename = "path")]
    Path(PathBuf),
}

#[derive(Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct SourceName(String);

impl Borrow<str> for SourceName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

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

impl Display for SourceName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.as_str().magenta().fmt(f)
    }
}

#[derive(Debug, PartialEq)]
pub struct SourcePath {
    pub name: SourceName,
    pub path: Option<PathBuf>,
}

impl Display for SourcePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.path {
            Some(path) => write!(f, "{}/{}", self.name, path.display()),
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

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        "a source name followed by an optional path".fmt(formatter)
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
