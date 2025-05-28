use std::{
    collections::{BTreeMap, BTreeSet, HashSet, VecDeque},
    fs,
    path::Path,
    process::Command,
    sync::OnceLock,
};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::{
    globs::Globs,
    module::Module,
    source::{hashable::HashableSource, name::SourceName},
    utils::pretty::Pretty,
};

use super::env::Env;

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Config {
    #[serde(default)]
    fetch: bool,

    #[serde(default = "Config::default_root_command")]
    root_command: Vec<String>,
    #[serde(default)]
    aliases: BTreeMap<String, Vec<String>>,

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
        Ok(fs::read_to_string(path)
            .ok()
            .map(|string| toml::from_str(&string))
            .transpose()
            .with_context(|| format!("Couldn't parse {}", path.pretty()))?
            .unwrap_or_default())
    }

    fn default_root_command() -> Vec<String> {
        vec!["sudo".to_string()]
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

    pub fn alias(&self, name: &str) -> Result<impl Iterator<Item = &str>> {
        let mut visited_aliases = HashSet::new();
        let command = self.aliases.get(name).unwrap().iter().map(|s| s.as_str());
        let mut command = VecDeque::from_iter(command);

        while let Some(cmd) = self.aliases.get(command[0]) {
            if visited_aliases.contains(cmd[0].as_str()) {
                bail!("Couldn't resolve alias `{name}` due to infinite loop in alias definitions");
            }
            visited_aliases.insert(cmd[0].as_str());
            command.pop_front();
            cmd.iter().rev().for_each(|arg| command.push_front(arg))
        }
        Ok(command.into_iter())
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

pub fn root_command() -> Option<Command> {
    let root_command = &config().root_command;
    (!root_command.is_empty()).then(|| {
        let mut command = Command::new(&root_command[0]);
        root_command.get(1..).map(|args| command.args(args));
        command
    })
}

pub fn alias(name: &str) -> Result<impl Iterator<Item = &str>> {
    config().alias(name)
}

pub fn aliases() -> impl Iterator<Item = (&'static str, impl Iterator<Item = &'static str>)> {
    config()
        .aliases
        .iter()
        .map(|(alias, command)| (alias.as_str(), command.iter().map(|s| s.as_str())))
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

#[cfg(test)]
mod tests {
    use super::*;
    use toml::toml;

    #[test]
    fn resolve_aliases() {
        let toml = toml! {
            [aliases]
            git = ["run", "git"]
            g = ["git"]
            gc = ["g", "commit"]
            ga = ["g", "commit", "--amend"]
            a = ["b"]
            b = ["a"]
        };
        let config = toml::from_str::<Config>(&toml.to_string()).unwrap();
        let aliases = [
            ("git", Ok(vec!["run", "git"])),
            ("g", Ok(vec!["run", "git"])),
            ("gc", Ok(vec!["run", "git", "commit"])),
            ("ga", Ok(vec!["run", "git", "commit", "--amend"])),
            ("a", Err(())),
            ("b", Err(())),
        ];
        for (alias, command) in aliases {
            let result = config.alias(alias);
            match command {
                Ok(command) => assert_eq!(command, result.unwrap().collect::<Vec<_>>()),
                Err(()) => assert!(result.is_err()),
            }
        }
    }
}
