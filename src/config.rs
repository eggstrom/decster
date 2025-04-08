use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    path::Path,
    sync::OnceLock,
};

use anyhow::{Context, Result, bail};
use crossterm::style::Stylize;
use globset::GlobSet;
use serde::Deserialize;

use crate::{
    cli::Behavior,
    module::Module,
    paths,
    source::{info::SourceInfo, name::SourceName},
    utils::{self, glob::GlobSetExt},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct Config {
    #[serde(skip)]
    behavior: Behavior,

    #[serde(skip, default)]
    sources: HashSet<SourceName>,
    #[serde(default, rename = "sources")]
    named_sources: HashMap<SourceName, SourceInfo>,
    #[serde(default)]
    modules: BTreeMap<String, Module>,
}

impl Config {
    fn load(behavior: Behavior) -> Result<Self> {
        let mut config = Config::parse(paths::config())?;
        config.behavior = behavior;
        config.load_modules(paths::modules())?;
        config.load_sources(paths::config_sources())?;
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
            if path.is_dir() {
                return Ok(());
            }
            if let Some(name) = rel_path.to_string_lossy().strip_suffix(".toml") {
                let module = Module::parse(path)?;
                if !self.modules.contains_key(name) {
                    self.modules.insert(name.to_string(), module);
                } else {
                    bail!("Module {} is defined twice", name.magenta());
                }
            }
            Ok(())
        })?;
        Ok(())
    }

    fn load_sources(&mut self, dir: &Path) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }
        for entry in fs::read_dir(dir)?.filter_map(Result::ok) {
            self.sources.insert(SourceName::from(entry.file_name()));
        }
        Ok(())
    }
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn load(behavior: Behavior) -> Result<()> {
    #[allow(clippy::ok_expect)] // The call to `ok` makes the output prettier.
    CONFIG
        .set(Config::load(behavior)?)
        .ok()
        .expect("`config::load` should only be called once");
    Ok(())
}

fn config() -> &'static Config {
    CONFIG
        .get()
        .expect("`config::load` should be called without failing before other functions in `config` are called")
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
    config().sources.contains(name)
}

pub fn has_named_source(name: &SourceName) -> bool {
    config().named_sources.contains_key(name)
}

pub fn named_source(name: &SourceName) -> Option<&'static SourceInfo> {
    config().named_sources.get(name)
}

pub fn module(name: &str) -> Option<&'static Module> {
    config().modules.get(name)
}

pub fn modules() -> impl Iterator<Item = (&'static str, &'static Module)> {
    config()
        .modules
        .iter()
        .map(|(name, module)| (name.as_str(), module))
}

pub fn modules_matching_globs(
    globs: &[String],
) -> Result<impl Iterator<Item = (&'static str, &'static Module)>, globset::Error> {
    let globs = GlobSet::from_globs(globs)?;
    Ok(modules().filter(move |(name, _)| globs.is_match(name)))
}
