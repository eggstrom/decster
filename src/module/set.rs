use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::PathBuf,
};

use anyhow::{Context, Result, bail};
use crossterm::style::Stylize;
use derive_more::From;
use indexmap::IndexMap;
use toml::Value;

use crate::{
    env::Env,
    fs::{mode::Mode, owner::OwnerIds},
    packages::PackageManager,
    state::State,
};

use super::{
    Module,
    link::{LinkKind, LinkMethod, ModuleLink},
    source::ModuleSource,
};

#[derive(From)]
pub struct ModuleSet<'a> {
    modules: IndexMap<&'a str, &'a Module>,
}

impl<'a> ModuleSet<'a> {
    pub fn links(
        &self,
        env: &mut Env,
    ) -> Result<impl ExactSizeIterator<Item = ModuleLink> + use<'_>> {
        let mut links = BTreeSet::new();
        for (_, module) in self.modules.iter() {
            let o = module.owner.as_ref().map(|o| o.ids(env)).transpose()?;
            let m = module.mode;
            Self::links_inner(o, m, &module.files, &mut links, LinkKind::File)?;
            Self::links_inner(o, m, &module.hard_links, &mut links, LinkKind::HardLink)?;
            Self::links_inner(o, m, &module.symlinks, &mut links, LinkKind::Symlink)?;
            Self::links_inner(o, m, &module.templates, &mut links, LinkKind::Template)?;
        }
        Ok(links.into_iter())
    }

    fn links_inner(
        owner: Option<OwnerIds>,
        mode: Option<Mode>,
        input: &'a BTreeMap<PathBuf, ModuleSource>,
        output: &mut BTreeSet<ModuleLink<'a>>,
        kind: LinkKind,
    ) -> Result<()> {
        for (path, source) in input.iter() {
            let link = ModuleLink::new(kind, path, source, owner, mode);
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

    fn packages(&self) -> BTreeMap<PackageManager, BTreeSet<String>> {
        let mut all_packages = BTreeMap::new();
        for (_, module) in &self.modules {
            for (manager, manager_packages) in &module.packages {
                let packages: &mut BTreeSet<_> = all_packages.entry(*manager).or_default();
                for package in manager_packages {
                    packages.insert(package.clone());
                }
            }
        }
        all_packages
    }

    pub fn enable(
        &self,
        env: &mut Env,
        state: &mut State,
        name: &str,
        method: LinkMethod,
    ) -> Result<()> {
        state.add_module(name, self.packages());
        for link in self.links(env)? {
            link.create(env, state, name, &self.context()?, method)
                .with_context(|| format!("Couldn't create link: {link}"))?
        }
        Ok(())
    }
}
