use std::{
    ffi::OsString,
    fmt::{self, Display, Formatter},
    ops::Deref,
    path::{Path, PathBuf},
    str::FromStr,
};

use bincode::{Decode, Encode};
use crossterm::style::Stylize;
use serde::Deserialize;
use thiserror::Error;

use crate::env::Env;

#[derive(Clone, Debug, Decode, Deserialize, Encode, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceName(String);

impl SourceName {
    pub fn static_path(&self, env: &Env) -> PathBuf {
        env.static_source_dir().join(self)
    }

    pub fn named_path(&self, env: &Env) -> PathBuf {
        env.named_source_dir().join(self)
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

impl From<OsString> for SourceName {
    fn from(value: OsString) -> Self {
        SourceName(value.to_string_lossy().into_owned())
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
