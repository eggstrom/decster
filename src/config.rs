use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use anyhow::{Result, anyhow};
use log::info;
use serde::Deserialize;

use crate::{
    link::{Link, LinkMethod},
    module::{Module, ModuleFilter},
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
        info!("Parsing config at {}", path.display());
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

    pub fn modules(
        &self,
        names: HashSet<String>,
        filter: ModuleFilter,
    ) -> impl Iterator<Item = (&str, &Module)> {
        self.modules.iter().filter_map(move |(name, module)| {
            ((names.is_empty() || names.contains(name))
                && match module.is_enabled(self.link_method) {
                    Err(_) => true,
                    Ok(enabled) => match filter {
                        ModuleFilter::All => true,
                        ModuleFilter::Enabled => enabled,
                        ModuleFilter::Disabled => !enabled,
                    },
                })
            .then_some((name.as_str(), module))
        })
    }

    pub fn links(&self, module: &str) -> Result<impl Iterator<Item = Link>> {
        self.modules
            .get(module)
            .map(|module| module.links(self.link_method))
            .ok_or(anyhow!("module `{module}` is not defined"))
    }
}
