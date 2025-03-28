use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
    path::{Path, PathBuf},
    str::FromStr,
};

use bincode::{Decode, Encode};
use crossterm::style::Stylize;
use serde::Deserialize;
use thiserror::Error;

use crate::paths;

#[derive(Clone, Debug, Decode, Deserialize, Encode, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceName(String);

impl SourceName {
    pub fn path(&self) -> PathBuf {
        paths::sources().join(self)
    }
}

impl Deref for SourceName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<Path> for SourceName {
    fn as_ref(&self) -> &Path {
        Path::new(&self.0)
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
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.as_str().magenta().fmt(f)
    }
}
