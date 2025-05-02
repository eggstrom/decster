use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use indexmap::IndexMap;
use serde::Deserialize;
use set::ModuleSet;
use source::ModuleSource;
use toml::Value;

use crate::{
    config,
    fs::{mode::Mode, owner::Owner},
    globs::Globs,
};

pub mod link;
pub mod set;
pub mod source;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Module {
    #[serde(default)]
    imports: Globs,

    #[serde(default)]
    owner: Option<Owner>,
    #[serde(default)]
    mode: Option<Mode>,

    #[serde(default)]
    files: BTreeMap<PathBuf, ModuleSource>,
    #[serde(default)]
    hard_links: BTreeMap<PathBuf, ModuleSource>,
    #[serde(default)]
    symlinks: BTreeMap<PathBuf, ModuleSource>,
    #[serde(default)]
    templates: BTreeMap<PathBuf, ModuleSource>,

    #[serde(default)]
    context: HashMap<String, Value>,
}

impl Module {
    pub fn parse<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        Ok(toml::from_str(&fs::read_to_string(path)?)?)
    }

    pub fn import<'a>(&'a self, name: &'a str) -> Result<ModuleSet<'a>> {
        let mut modules = IndexMap::from([(name, self)]);
        Self::import_inner(&mut modules, &self.imports)?;
        Ok(ModuleSet { modules })
    }

    fn import_inner(modules: &mut IndexMap<&str, &Module>, imports: &Globs) -> Result<()> {
        for (name, module) in config::modules_matching_globs(imports) {
            if !modules.contains_key(name) {
                modules.insert(name, module);
                Self::import_inner(modules, &module.imports)?;
            }
        }
        Ok(())
    }
}
