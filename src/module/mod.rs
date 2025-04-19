use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use serde::Deserialize;
use source::ModuleSource;
use toml::Value;

use crate::env::User;

pub mod link;
pub mod set;
pub mod source;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Module {
    #[serde(default)]
    pub imports: HashSet<String>,
    #[serde(default)]
    pub user: Option<String>,

    #[serde(default)]
    files: BTreeMap<PathBuf, ModuleSource>,
    #[serde(default)]
    hard_links: BTreeMap<PathBuf, ModuleSource>,
    #[serde(default)]
    symlinks: BTreeMap<PathBuf, ModuleSource>,
    #[serde(default)]
    templates: BTreeMap<PathBuf, ModuleSource>,

    #[serde(default)]
    pub context: HashMap<String, Value>,
}

impl Module {
    pub fn parse<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        Ok(toml::from_str(&fs::read_to_string(path)?)?)
    }

    pub fn user(&self) -> Result<Option<User>> {
        Ok(self
            .user
            .as_ref()
            .map(|user| User::new(user))
            .transpose()?
            .filter(|user| !user.is_current()))
    }
}
