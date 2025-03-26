use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

use serde::Deserialize;

pub mod name;
pub mod path;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum Source {
    Text(String),
    Path(PathBuf),
}

impl Display for Source {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Source::Text(_) => "text",
            Source::Path(_) => "path",
        }
        .fmt(f)
    }
}
