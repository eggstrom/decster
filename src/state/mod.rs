use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use bincode::{Decode, Encode, config::Configuration};
use crossterm::style::Stylize;
use globset::GlobSet;
use path::PathInfo;

use crate::{
    module::set::ModuleSet,
    paths,
    source::{hashable::HashableSource, ident::SourceIdent},
    users::Users,
    utils::{
        glob::GlobSetExt,
        pretty::{PrettyPathExt, PrettyStrSliceExt},
    },
};

pub mod path;

#[derive(Decode, Default, Encode)]
pub struct State {
    sources: BTreeMap<SourceIdent, HashableSource>,
    module_paths: BTreeMap<String, Vec<(PathBuf, PathInfo)>>,
    paths: HashSet<PathBuf>,
}

impl State {
    pub fn load() -> Result<Self> {
        let dir = paths::named_sources();
        fs::create_dir_all(dir)
            .with_context(|| format!("Couldn't create path: {}", dir.pretty()))?;
        Ok(File::open(paths::state())
            .ok()
            .and_then(|mut file| bincode::decode_from_std_read(&mut file, Self::bin_config()).ok())
            .unwrap_or_default())
    }

    pub fn save(&self) -> Result<()> {
        let mut file = File::create(paths::state())?;
        bincode::encode_into_std_write(self, &mut file, Self::bin_config())?;
        Ok(())
    }

    fn bin_config() -> Configuration {
        bincode::config::standard()
    }

    pub fn is_source_fetched(&self, ident: &SourceIdent, source: &HashableSource) -> bool {
        self.sources.get(ident).is_some_and(|s| s == source) && ident.path().exists()
    }

    pub fn is_module_enabled(&self, module: &str) -> bool {
        self.module_paths.contains_key(module)
    }

    pub fn is_path_owned<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        self.paths.contains(path.as_ref())
    }

    pub fn add_module(&mut self, name: &str) {
        if !self.module_paths.contains_key(name) {
            self.module_paths.insert(name.to_string(), Vec::new());
        }
    }

    pub fn add_source(&mut self, ident: &SourceIdent, source: &HashableSource) {
        self.sources.insert(ident.clone(), source.clone());
    }

    pub fn add_path(&mut self, module: &str, path: &Path, info: PathInfo) {
        self.paths.insert(path.to_path_buf());
        if let Some(paths) = self.module_paths.get_mut(module) {
            paths.push((path.to_path_buf(), info));
        } else {
            self.module_paths
                .insert(module.into(), vec![(path.to_path_buf(), info)]);
        }
    }

    pub fn owned_paths(&self) -> impl ExactSizeIterator<Item = (&str, &[(PathBuf, PathInfo)])> {
        self.module_paths
            .iter()
            .map(|(name, paths)| (name.as_str(), paths.as_slice()))
    }

    pub fn modules_matching_globs(&self, globs: &[String]) -> Result<Vec<String>> {
        let glob_set = GlobSet::from_globs(globs)?;
        let matches: Vec<_> = self
            .module_paths
            .keys()
            .filter(move |name| glob_set.is_match(name))
            .cloned()
            .collect();
        if matches.is_empty() {
            bail!("{} didn't match any enabled modules", globs.pretty());
        }
        Ok(matches)
    }

    pub fn enable_module(
        &mut self,
        users: &mut Users,
        name: &str,
        modules: &ModuleSet,
    ) -> Result<()> {
        if let Err(err) = modules
            .enable(users, self, name)
            .with_context(|| format!("Couldn't enable module {}", name.magenta()))
        {
            if let Err(err) = self.disable_module(name) {
                eprintln!("{} {err:?}", "error:".red());
            }
            bail!(err);
        }
        Ok(())
    }

    fn disable_module(&mut self, name: &str) -> Result<()> {
        let paths = self
            .module_paths
            .get_mut(name)
            .expect("Whether `name` exists should be checked before calling this method");
        // Paths are removed in reverse order to make sure directories are
        // removed last.
        for i in (0..paths.len()).rev() {
            let (path, info) = &paths[i];
            info.remove_if_owned(path)?;
            self.paths.remove(path);
            paths.remove(i);
        }
        self.module_paths.remove(name);
        println!("Disabled {}", name.magenta());
        Ok(())
    }

    pub fn disable_modules_matching_globs(&mut self, globs: &[String]) -> Result<()> {
        for name in self.modules_matching_globs(globs)? {
            if let Err(err) = self.disable_module(&name) {
                eprintln!("{} {err:?}", "error:".red());
            }
        }
        Ok(())
    }

    fn can_update(&self) -> Result<()> {
        if self.module_paths.is_empty() {
            bail!("There are no enabled modules to update");
        }
        Ok(())
    }

    fn update_module(
        &mut self,
        users: &mut Users,
        name: &str,
        modules: Option<&ModuleSet>,
    ) -> Result<()> {
        self.disable_module(name)?;
        if let Some(modules) = modules {
            self.enable_module(users, name, modules)?;
        }
        println!("Updated {}", name.magenta());
        Ok(())
    }

    pub fn update_all_modules(&mut self, users: &mut Users) -> Result<()> {
        self.can_update()?;
        for name in self.module_paths.keys().cloned().collect::<Vec<_>>() {
            if let Err(err) = self.update_module(users, &name, ModuleSet::new(&name).ok().as_ref())
            {
                eprintln!("{} {err:?}", "error:".red());
            }
        }
        Ok(())
    }

    pub fn update_modules_matching_globs(
        &mut self,
        users: &mut Users,
        globs: &[String],
    ) -> Result<()> {
        self.can_update()?;
        for name in self.modules_matching_globs(globs)? {
            if let Err(err) = self.update_module(users, &name, ModuleSet::new(&name).ok().as_ref())
            {
                eprintln!("{} {err:?}", "error:".red());
            }
        }
        Ok(())
    }
}
