use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::Path,
    sync::OnceLock,
};

use anyhow::{Result, bail};
use crossterm::style::Stylize;
use serde::Deserialize;

use crate::{
    cli::{Behavior, Cli},
    module::Module,
    paths,
    source::{Source, name::SourceName},
    utils,
};

#[derive(Deserialize)]
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
    pub fn load(cli: &Cli) -> Result<Self> {
        let config_dir = cli.config.as_deref().unwrap_or(paths::config());
        let mut config = Config::parse(config_dir.join("config.toml"))?;
        config.behavior = cli.behavior.clone();
        config.load_modules(&config_dir.join("modules"))?;
        Ok(config)
    }

    fn parse<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        Ok(toml::from_str(&fs::read_to_string(path)?)?)
    }

    fn load_modules(&mut self, dir: &Path) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }
        utils::fs::walk_dir_with_rel(dir, false, |path, rel_path| {
            if path.is_dir() {
                return Ok(());
            }
            if let Some(name) = rel_path.to_string_lossy().strip_suffix(".toml") {
                let module = Module::parse(path)?;
                if self.modules.insert(name.to_string(), module).is_some() {
                    bail!("Module {} is defined twice", name.magenta());
                }
            }
            Ok(())
        })?;
        Ok(())
    }
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn load(cli: &Cli) -> Result<()> {
    #[allow(clippy::ok_expect)] // The call to `ok` makes the output prettier.
    CONFIG
        .set(Config::load(cli)?)
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

pub fn source(name: &SourceName) -> Option<&'static Source> {
    config().sources.get(name)
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
