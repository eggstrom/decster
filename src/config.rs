use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    sync::OnceLock,
};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::{
    globs::Globs,
    module::Module,
    source::{hashable::HashableSource, name::SourceName},
    utils::pretty::Pretty,
};

use super::env::Env;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Config {
    #[serde(default)]
    fetch: bool,

    #[serde(default)]
    aliases: BTreeMap<String, String>,

    #[serde(skip, default)]
    modules: BTreeMap<String, Module>,
    #[serde(skip, default)]
    static_sources: BTreeSet<SourceName>,
    #[serde(skip, default)]
    dynamic_sources: BTreeMap<SourceName, HashableSource>,
}

impl Config {
    pub fn load(env: &Env) -> Result<Self> {
        let mut config = Config::parse(env.config_file())?;
        config.load_modules(env.module_dir())?;
        config.load_static_sources(env.static_source_dir())?;
        config.load_dynamic_sources(env.dynamic_source_file())?;
        Ok(config)
    }

    fn parse<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let text = fs::read_to_string(path)
            .with_context(|| format!("Couldn't read config at {}", path.pretty()))?;
        Ok(toml::from_str(&text)?)
    }

    fn load_modules(&mut self, dir: &Path) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }
        crate::fs::walk_dir_rel(dir, false, false, |path, rel_path| {
            if path.is_file() {
                if let Some(name) = rel_path.to_string_lossy().strip_suffix(".toml") {
                    let module = Module::parse(path)?;
                    self.modules.insert(name.to_string(), module);
                }
            }
            Ok(())
        })?;
        Ok(())
    }

    fn load_static_sources(&mut self, dir: &Path) -> Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)?.filter_map(Result::ok) {
                self.static_sources
                    .insert(SourceName::from(entry.file_name()));
            }
        }
        Ok(())
    }

    fn load_dynamic_sources(&mut self, file: &Path) -> Result<()> {
        if file.is_file() {
            self.dynamic_sources = toml::from_str(&fs::read_to_string(file)?)?;
        }
        Ok(())
    }
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn load(env: &Env) -> Result<()> {
    CONFIG
        .set(Config::load(env)?)
        .ok()
        .expect("`config::load` should only be called once");
    Ok(())
}

fn config() -> &'static Config {
    CONFIG
        .get()
        .expect("`config::load` should be called without failing before config is used")
}

pub fn fetch() -> bool {
    config().fetch
}

pub fn aliases() -> impl Iterator<Item = (&'static str, &'static str)> {
    config()
        .aliases
        .iter()
        .map(|(alias, command)| (alias.as_str(), command.as_str()))
}

pub fn has_static_source(name: &SourceName) -> bool {
    config().static_sources.contains(name)
}

pub fn dynamic_source(name: &SourceName) -> Option<&'static HashableSource> {
    config().dynamic_sources.get(name)
}

pub fn static_sources() -> impl ExactSizeIterator<Item = &'static SourceName> {
    config().static_sources.iter()
}

pub fn static_sources_matching_globs(globs: &Globs) -> impl Iterator<Item = &'static SourceName> {
    static_sources().filter(move |name| globs.is_match(name))
}

pub fn module(name: &str) -> Option<(&'static str, &'static Module)> {
    config()
        .modules
        .get_key_value(name)
        .map(|(name, module)| (name.as_str(), module))
}

pub fn modules() -> impl Iterator<Item = (&'static str, &'static Module)> {
    config()
        .modules
        .iter()
        .map(|(name, module)| (name.as_str(), module))
}

pub fn modules_matching_globs(
    globs: &Globs,
) -> impl Iterator<Item = (&'static str, &'static Module)> {
    modules().filter(move |(name, _)| globs.is_match(name))
}
