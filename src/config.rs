use std::{collections::HashMap, fs, path::Path};

use anyhow::{Result, anyhow};
use serde::Deserialize;

use crate::{
    link::{Link, LinkMethod},
    module::Module,
    source::{Source, SourceName},
};

#[derive(Deserialize)]
pub struct Config {
    #[serde(default)]
    link_method: LinkMethod,
    #[serde(default)]
    sources: HashMap<SourceName, Source>,
    #[serde(default)]
    modules: HashMap<String, Module>,
}

impl Config {
    pub fn parse(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().join("config.toml");
        Ok(toml::from_str(&fs::read_to_string(path)?)?)
    }

    pub fn source(&self, name: &str) -> Result<&Source> {
        self.sources
            .get(name)
            .ok_or(anyhow!("source `{name}` is not defined"))
    }

    pub fn links(&self, module: &str) -> Option<impl Iterator<Item = Link>> {
        self.modules.get(module).map(|module| {
            module
                .links()
                .map(|(link, method)| link.with_method(method.unwrap_or(self.link_method)))
        })
    }

    pub fn module_names(&self) -> impl Iterator<Item = &str> {
        self.modules.keys().map(|s| s.as_str())
    }
}
