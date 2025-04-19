use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    rc::Rc,
};

use anyhow::{Result, bail};
use indexmap::IndexMap;

use crate::{config, env::User, state::State};

use super::{Module, link::ModuleLink, source::ModuleSource};

pub struct ModuleSet {
    modules: Vec<(&'static str, &'static Module)>,
}

impl ModuleSet {
    pub fn new(name: &str) -> Result<Self> {
        let mut modules = IndexMap::new();
        Self::new_inner(name, &mut modules)?;
        Ok(ModuleSet {
            modules: modules.into_iter().collect(),
        })
    }

    fn new_inner(name: &str, modules: &mut IndexMap<&str, &Module>) -> Result<()> {
        if !modules.contains_key(name) {
            let (name, module) = config::module(name)?;
            modules.insert(name, module);
            for import in config::modules_matching_globs(&module.imports)? {
                Self::new_inner(import, modules)?;
            }
        }
        Ok(())
    }

    pub fn links(&self) -> Result<impl ExactSizeIterator<Item = ModuleLink> + use<'_>> {
        let mut links = BTreeSet::new();
        for (_, module) in self.modules.iter() {
            let user = module.user()?.map(Rc::new);
            let user = user.as_ref();
            Self::links_inner(user, &module.files, &mut links, ModuleLink::file)?;
            Self::links_inner(user, &module.hard_links, &mut links, ModuleLink::hard_link)?;
            Self::links_inner(user, &module.symlinks, &mut links, ModuleLink::symlink)?;
        }
        Ok(links.into_iter())
    }

    fn links_inner<F>(
        user: Option<&Rc<User>>,
        input: &'static BTreeMap<PathBuf, ModuleSource>,
        output: &mut BTreeSet<ModuleLink<'static>>,
        f: F,
    ) -> Result<()>
    where
        F: Fn(&'static Path, &'static ModuleSource, Option<&Rc<User>>) -> ModuleLink<'static>,
    {
        for (path, source) in input.iter() {
            let link = f(path, source, user);
            if !output.insert(link) {
                bail!("Path {} is used multiple times", path.display());
            }
        }
        Ok(())
    }

    pub fn enable(&self, state: &mut State, name: &str) -> Result<()> {
        state.add_module(name);
        for link in self.links()? {
            link.create(state, name)?
        }
        Ok(())
    }
}
