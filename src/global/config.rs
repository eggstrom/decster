use std::{
    collections::{BTreeMap, HashMap},
    fs,
};

use anyhow::Result;
use serde::Deserialize;

use crate::{
    cli::{Behavior, Cli},
    module::Module,
    source::{Source, name::SourceName},
};

use super::paths::Paths;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(super) struct Config {
    #[serde(skip)]
    pub behavior: Behavior,

    #[serde(default)]
    pub sources: HashMap<SourceName, Source>,
    #[serde(default)]
    pub modules: BTreeMap<String, Module>,
}

impl Config {
    pub fn parse(cli: &Cli, paths: &Paths) -> Result<Self> {
        let path = cli
            .config
            .as_deref()
            .unwrap_or(&paths.config)
            .join("config.toml");
        Ok(toml::from_str(&fs::read_to_string(path)?)?)
    }
}

fn config() -> &'static Config {
    &super::state().config
}

pub fn quiet() -> bool {
    config().behavior.quiet
}

pub fn dry_run() -> bool {
    config().behavior.dry_run
}

pub fn force() -> bool {
    config().behavior.force
}

pub fn source(name: &SourceName) -> Option<&Source> {
    config().sources.get(name)
}

pub fn module(name: &str) -> Option<&Module> {
    config().modules.get(name)
}

pub fn modules() -> impl Iterator<Item = (&'static str, &'static Module)> {
    config()
        .modules
        .iter()
        .map(|(name, module)| (name.as_str(), module))
}
