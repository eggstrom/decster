use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::{Path, PathBuf},
    rc::Rc,
};

use anyhow::{Context, Result, bail};
use crossterm::style::Stylize;
use indexmap::IndexMap;
use toml::Value;

use crate::{
    global::{config, env::User},
    state::State,
};

use super::{Module, link::ModuleLink, source::ModuleSource};

pub struct ModuleSet {
    modules: IndexMap<&'static str, &'static Module>,
}

impl ModuleSet {
    pub fn new(name: &str) -> Result<Self> {
        let mut modules = IndexMap::new();
        Self::new_inner(name, &mut modules)?;
        Ok(ModuleSet { modules })
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
            Self::links_inner(user, &module.templates, &mut links, ModuleLink::template)?;
        }
        Ok(links.into_iter())
    }

    fn links_inner(
        user: Option<&Rc<User>>,
        input: &'static BTreeMap<PathBuf, ModuleSource>,
        output: &mut BTreeSet<ModuleLink<'static>>,
        f: fn(&'static Path, &'static ModuleSource, Option<&Rc<User>>) -> ModuleLink<'static>,
    ) -> Result<()> {
        for (path, source) in input.iter() {
            let link = f(path, source, user);
            if !output.insert(link) {
                bail!("Path {} is used multiple times", path.display());
            }
        }
        Ok(())
    }

    fn context(&self) -> Result<HashMap<&str, &Value>> {
        let mut context = HashMap::new();
        for module in self.modules.values() {
            for (name, value) in module.context.iter() {
                let name = name.as_str();
                if context.insert(name, value).is_some() {
                    let name = name.magenta();
                    bail!("Variable {name} is defined in multiple contexts",);
                }
            }
        }
        Ok(context)
    }

    pub fn enable(&self, state: &mut State, name: &str) -> Result<()> {
        state.add_module(name);
        for link in self.links()? {
            link.create(state, name, &self.context()?)
                .with_context(|| format!("Couldn't create link: {link}"))?
        }
        Ok(())
    }
}
