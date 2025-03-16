use std::{collections::HashMap, path::Path};

use anyhow::Result;
use serde::Deserialize;

use crate::{
    module::{Method, Module},
    source::{Source, SourceName},
};

#[derive(Deserialize)]
pub struct Config {
    #[serde(default)]
    method: Method,
    #[serde(default)]
    sources: HashMap<SourceName, Source>,
    #[serde(default)]
    modules: HashMap<String, Module>,
}

impl Config {
    pub fn parse(path: impl AsRef<Path>) -> Result<Self> {
        todo!()
    }

    pub fn module_names(&self) -> impl Iterator<Item = &str> {
        self.modules.keys().map(|s| s.as_str())
    }
}
