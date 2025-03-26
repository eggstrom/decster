use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use anyhow::{Result, anyhow};
use crossterm::style::Stylize;
use serde::Deserialize;

use crate::{
    module::{Module, ModuleFilter},
    paths,
    source::{Source, name::SourceName},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
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

    pub fn source(&self, name: &SourceName) -> Result<&Source> {
        self.sources
            .get(name)
            .ok_or(anyhow!("Couldn't find source: {}", name.magenta()))
    }

    pub fn modules(
        &self,
        names: HashSet<String>,
        filter: ModuleFilter,
    ) -> impl Iterator<Item = (&str, &Module)> {
        self.modules.iter().filter_map(move |(name, module)| {
            ((names.is_empty() || names.contains(name))
                && match filter {
                    ModuleFilter::All => true,
                    ModuleFilter::Enabled => true,
                    ModuleFilter::Disabled => true,
                })
            .then_some((name.as_str(), module))
        })
    }
}
