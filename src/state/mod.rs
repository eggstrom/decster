use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use bincode::{Decode, Encode, config::Configuration};
use crossterm::style::Stylize;
use path::PathInfo;

use crate::{
    env::Env, globs::Globs, module::set::ModuleSet, source::{hashable::HashableSource, ident::SourceIdent}, utils::pretty::Pretty
};

pub mod path;

#[derive(Decode, Default, Encode)]
pub struct State {
    sources: BTreeMap<SourceIdent, HashableSource>,
    module_paths: BTreeMap<String, Vec<(PathBuf, PathInfo)>>,
    paths: HashSet<PathBuf>,
}

impl State {
    pub fn load(env: &Env) -> Result<Self> {
        let dir = env.named_sources();
        fs::create_dir_all(dir)
            .with_context(|| format!("Couldn't create path: {}", env.tildefy(dir).pretty()))?;
        Ok(File::open(env.state())
            .ok()
            .and_then(|mut file| bincode::decode_from_std_read(&mut file, Self::bin_config()).ok())
            .unwrap_or_default())
    }

    pub fn save(&self, env: &Env) -> Result<()> {
        let mut file = File::create(env.state())?;
        bincode::encode_into_std_write(self, &mut file, Self::bin_config())?;
        Ok(())
    }

    fn bin_config() -> Configuration {
        bincode::config::standard()
    }

    pub fn sources(&self) -> impl ExactSizeIterator<Item = &SourceIdent> {
        self.sources.keys()
    }

    pub fn sources_matching_globs(&self, globs: &Globs) -> impl Iterator<Item = &SourceIdent> {
        self.sources()
            .filter(move |ident| ident.is_named_and(|name| globs.is_match(name)))
    }

    pub fn is_source_fetched(
        &self,
        env: &Env,
        ident: &SourceIdent,
        source: &HashableSource,
    ) -> bool {
        self.sources
            .get(ident)
            .is_some_and(|s| s == source && source.check(&ident.path(env)).is_ok())
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

    pub fn modules(&self) -> Vec<String> {
        self.module_paths.keys().cloned().collect()
    }

    pub fn modules_matching_globs(&self, globs: &Globs) -> Vec<String> {
        self.module_paths
            .keys()
            .filter(move |name| globs.is_match(name))
            .cloned()
            .collect()
    }

    pub fn enable_module(&mut self, env: &Env, name: &str, modules: ModuleSet) -> Result<()> {
        if let Err(err) = modules
            .enable(env, self, name)
            .with_context(|| format!("Couldn't enable module {}", name.magenta()))
        {
            if let Err(err) = self.disable_module(env, name) {
                eprintln!("{} {err:?}", "error:".red());
            }
            bail!(err);
        }
        Ok(())
    }

    pub fn disable_module(&mut self, env: &Env, module: &str) -> Result<()> {
        let paths = self
            .module_paths
            .get_mut(module)
            .expect("Whether `name` exists should be checked before calling this method");
        // Paths are removed in reverse order to make sure directories are
        // removed last.
        for i in (0..paths.len()).rev() {
            let (path, info) = &paths[i];
            info.remove_if_owned(env, path)?;
            self.paths.remove(path);
            paths.remove(i);
        }
        self.module_paths.remove(module);
        Ok(())
    }

    pub fn update_module(
        &mut self,
        env: &Env,
        name: &str,
        modules: Option<ModuleSet>,
    ) -> Result<()> {
        self.disable_module(env, name)?;
        if let Some(modules) = modules {
            self.enable_module(env, name, modules)?;
        }
        Ok(())
    }
}
