use std::path::PathBuf;

use serde::Deserialize;

pub mod name;
pub mod path;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum Source {
    Text(String),
    Path(PathBuf),
}
