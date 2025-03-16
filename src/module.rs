use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use serde::Deserialize;

use crate::source::SourcePath;

#[derive(Debug, Default, Deserialize, PartialEq)]
pub enum Method {
    #[serde(rename = "copy")]
    Copy,
    #[default]
    #[serde(rename = "soft-link")]
    SoftLink,
    #[serde(rename = "hard-link")]
    HardLink,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Module {
    import: HashSet<String>,
    method: Option<Method>,
    #[serde(default)]
    files: HashMap<PathBuf, SourcePath>,
}
