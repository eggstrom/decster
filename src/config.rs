use std::{collections::HashMap, fs, path::Path};

use anyhow::{Result, anyhow};
use serde::Deserialize;

use crate::{
    link::{Link, LinkMethod},
    module::Module,
    paths,
    source::{Source, SourceName},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(default)]
    pub link_method: LinkMethod,
    #[serde(default)]
    sources: HashMap<SourceName, Source>,
    #[serde(default)]
    modules: HashMap<String, Module>,
}

impl Config {
    pub fn parse(path: Option<&Path>) -> Result<Self> {
        let path = path.unwrap_or(paths::config()?).join("config.toml");
        Ok(toml::from_str(&fs::read_to_string(path)?)?)
    }

    pub fn source(&self, name: &str) -> Result<&Source> {
        self.sources
            .get(name)
            .ok_or(anyhow!("source `{name}` is not defined"))
    }

    pub fn module(&self, name: &str) -> Result<&Module> {
        self.modules
            .get(name)
            .ok_or(anyhow!("module `{name}` is not defined"))
    }

    pub fn modules<I>(&self) -> impl Iterator<Item = (&String, &Module)> {
        self.modules.iter()
    }

    pub fn links(&self, module: &str) -> Result<impl Iterator<Item = Link>> {
        self.modules
            .get(module)
            .map(|module| {
                module
                    .links()
                    .map(|(link, method)| link.with_method(method.unwrap_or(self.link_method)))
            })
            .ok_or(anyhow!("module `{module}` is not defined"))
    }
}
