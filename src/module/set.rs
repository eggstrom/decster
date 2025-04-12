use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};
use indexmap::IndexMap;
use nix::unistd::Uid;

use crate::{config, state::State, users::Users};

use super::{Module, link::ModuleLink, source::ModuleSource};

pub struct ModuleSet {
    modules: Vec<&'static Module>,
}

impl ModuleSet {
    pub fn new(name: &str) -> Result<Self> {
        let mut modules = IndexMap::new();
        Self::new_inner(name, &mut modules)?;
        Ok(ModuleSet {
            modules: modules.into_values().collect(),
        })
    }

    fn new_inner(name: &str, modules: &mut IndexMap<&str, &Module>) -> Result<()> {
        if !modules.contains_key(name) {
            let (name, module) = config::module(name)?;
            modules.insert(name, module);
            for import in module.imports.iter() {
                Self::new_inner(import, modules)?;
            }
        }
        Ok(())
    }

    pub fn links(
        &self,
        users: &mut Users,
    ) -> Result<impl ExactSizeIterator<Item = ModuleLink> + use<'_>> {
        let mut links = BTreeSet::new();
        for module in self.modules.iter() {
            let uid = module.user.as_ref().map(|u| users.uid(u)).transpose()?;
            Self::links_inner(uid, &module.files, &mut links, ModuleLink::file)?;
            Self::links_inner(uid, &module.hard_links, &mut links, ModuleLink::hard_link)?;
            Self::links_inner(uid, &module.symlinks, &mut links, ModuleLink::symlink)?;
        }
        Ok(links.into_iter())
    }

    fn links_inner<F>(
        uid: Option<Uid>,
        input: &'static BTreeMap<PathBuf, ModuleSource>,
        output: &mut BTreeSet<ModuleLink<'static>>,
        f: F,
    ) -> Result<()>
    where
        F: Fn(&'static Path, &'static ModuleSource, Option<Uid>) -> ModuleLink<'static>,
    {
        for (path, source) in input.iter() {
            let link = f(path, source, uid);
            if !output.insert(link) {
                bail!("Path {} is used multiple times", path.display());
            }
        }
        Ok(())
    }

    pub fn enable(&self, users: &mut Users, state: &mut State, name: &str) -> Result<()> {
        for link in self.links(users)? {
            link.create(users, state, name)?
        }
        Ok(())
    }
}
