use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::Path,
};

use anyhow::Result;
use crossterm::style::Stylize;
use serde::Deserialize;

use crate::{
    module::Module,
    paths,
    source::{Source, name::SourceName},
    state::State,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Config {
    #[serde(default)]
    sources: HashMap<SourceName, Source>,
    #[serde(default)]
    modules: BTreeMap<String, Module>,
}

impl Config {
    pub fn parse(path: Option<&Path>) -> Result<Self> {
        let path = path.unwrap_or(paths::config()).join("config.toml");
        Ok(toml::from_str(&fs::read_to_string(path)?)?)
    }

    pub fn source(&self, name: &SourceName) -> Option<&Source> {
        self.sources.get(name)
    }

    pub fn module(&self, name: &str) -> Option<&Module> {
        self.modules.get(name)
    }

    pub fn modules(&self) -> impl Iterator<Item = (&str, &Module)> {
        self.modules
            .iter()
            .map(|(name, module)| (name.as_str(), module))
    }

    pub fn enable_module(&self, state: &mut State, name: &str) {
        if state.is_module_enabled(name) {
            println!("Module {} is already enabled", name.magenta());
        } else if let Some(module) = self.module(name) {
            self.enable_module_inner(state, name, module);
        } else {
            println!("Module {} isn't defined", name.magenta());
        }
    }

    pub fn enable_all_modules(&self, state: &mut State) {
        let mut has_enabled = false;
        for (name, module) in self.modules() {
            if !state.is_module_enabled(name) {
                self.enable_module_inner(state, name, module);
                has_enabled = true;
            }
        }
        if !has_enabled {
            println!("There are no disabled modules");
        }
    }

    fn enable_module_inner(&self, state: &mut State, name: &str, module: &Module) {
        println!("Enabling module {}", name.magenta());
        module.add_sources(&self, state);
        module.create_files(state, name);
        module.create_hard_links(state, name);
        module.create_symlinks(state, name);
    }
}
