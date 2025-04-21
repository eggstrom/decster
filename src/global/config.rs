use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result, anyhow};
use globset::GlobSet;
use serde::Deserialize;

use crate::{
    cli::Behavior,
    module::Module,
    source::{hashable::HashableSource, name::SourceName},
    utils::{self, glob::GlobSetExt},
};

use super::env::Env;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(super) struct Config {
    #[serde(skip)]
    behavior: Behavior,

    #[serde(skip, default)]
    modules: BTreeMap<String, Module>,
    #[serde(skip, default)]
    static_sources: HashSet<SourceName>,
    #[serde(skip, default)]
    dynamic_sources: HashMap<SourceName, HashableSource>,
}

impl Config {
    pub fn load(env: &Env, behavior: Behavior) -> Result<Self> {
        let mut config = Config::parse(&env.config)?;
        config.behavior = behavior;
        config.load_modules(&env.modules)?;
        config.load_static_sources(&env.static_sources)?;
        config.load_dynamic_sources(&env.dynamic_sources)?;
        Ok(config)
    }

    fn parse<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let text = fs::read_to_string(path)
            .with_context(|| format!("Couldn't read config at {}", path.display()))?;
        Ok(toml::from_str(&text)?)
    }

    fn load_modules(&mut self, dir: &Path) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }
        utils::fs::walk_dir_rel(dir, false, false, |path, rel_path| {
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

fn config() -> &'static Config {
    &super::state().config
}

pub fn fetch() -> bool {
    config().behavior.fetch
}

pub fn overwrite() -> bool {
    config().behavior.overwrite
}

pub fn dry_run() -> bool {
    config().behavior.dry_run
}

pub fn quiet() -> bool {
    config().behavior.quiet
}

pub fn has_source(name: &SourceName) -> bool {
    config().static_sources.contains(name)
}

pub fn dynamic_source(name: &SourceName) -> Option<&'static HashableSource> {
    config().dynamic_sources.get(name)
}

pub fn module(name: &str) -> Result<(&'static str, &'static Module)> {
    config()
        .modules
        .get_key_value(name)
        .map(|(name, module)| (name.as_str(), module))
        .ok_or(anyhow!("Module isn't defined"))
}

pub fn modules() -> impl Iterator<Item = (&'static str, &'static Module)> {
    config()
        .modules
        .iter()
        .map(|(name, module)| (name.as_str(), module))
}

pub fn modules_matching_globs<I>(
    globs: I,
) -> Result<impl Iterator<Item = &'static str>, globset::Error>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let glob_set = GlobSet::from_globs(globs)?;
    Ok(modules().filter_map(move |(name, _)| glob_set.is_match(name).then_some(name)))
}
