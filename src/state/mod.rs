use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File},
    io, mem,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use bincode::{Decode, Encode, config::Configuration};
use crossterm::style::Stylize;
use path::PathInfo;

use crate::{
    global::{config, paths},
    module::Module,
    out,
    source::{Source, name::SourceName},
    utils::{self, output::PathExt},
};

pub mod path;

#[derive(Decode, Default, Encode)]
pub struct State {
    module_paths: BTreeMap<String, Vec<(PathBuf, PathInfo)>>,
    paths: HashSet<PathBuf>,
}

impl State {
    pub fn load() -> Result<Self> {
        fs::create_dir_all(paths::sources())
            .with_context(|| format!("Couldn't create path: {}", paths::sources().display_dir()))?;
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

    pub fn is_module_enabled(&self, module: &str) -> bool {
        self.module_paths.get(module).is_some()
    }

    pub fn is_path_used<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        self.paths.contains(path.as_ref())
    }

    pub fn add_source(&self, name: &SourceName, source: &Source) -> io::Result<()> {
        let source_path = paths::sources().join(name);
        if source_path.exists() {
            utils::fs::remove_all(&source_path)?;
        }

        match source {
            Source::Text(text) => self.add_text_source(&source_path, text)?,
            Source::Path(path) => self.add_path_source(&source_path, path)?,
        }
        Ok(())
    }

    fn add_text_source(&self, source_path: &Path, text: &str) -> io::Result<()> {
        fs::write(&source_path, text)
    }

    fn add_path_source(&self, source_path: &Path, path: &Path) -> io::Result<()> {
        utils::fs::copy_all(path, &source_path)
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

    pub fn create_dir(&mut self, module: &str, path: &Path) -> io::Result<()> {
        if !path.is_dir() {
            fs::create_dir(path)?;
            self.add_path(module, path, PathInfo::Directory);
        }
        Ok(())
    }

    /// Returns a list of module names, definitions, and owned paths, based on
    /// the provided filter.
    pub fn modules(
        &self,
        modules: HashSet<String>,
        filter: ModuleFilter,
    ) -> impl Iterator<Item = (&str, &Module, Option<&Vec<(PathBuf, PathInfo)>>)> {
        config::modules()
            .map(|(name, module)| (name, module, self.module_paths.get(name)))
            .filter(move |(name, _, paths)| {
                (modules.is_empty() || modules.contains(*name))
                    && match filter {
                        ModuleFilter::All => true,
                        ModuleFilter::Enabled => paths.is_some(),
                        ModuleFilter::Disabled => paths.is_none(),
                    }
            })
    }

    pub fn enable_module(&mut self, name: &str) {
        if self.is_module_enabled(name) {
            out!(0, "", "Module {} isn't disabled", name.magenta());
        } else if let Some(module) = config::module(name) {
            self.enable_module_inner(name, module);
        } else {
            out!(0, "", "Module {} isn't defined", name.magenta());
        }
    }

    pub fn enable_all_modules(&mut self) {
        let mut has_enabled = false;
        for (name, module) in config::modules() {
            if !self.is_module_enabled(name) {
                self.enable_module_inner(name, module);
                has_enabled = true;
            }
        }
        if !has_enabled {
            out!(0, "", "There are no disabled modules");
        }
    }

    fn enable_module_inner(&mut self, name: &str, module: &Module) {
        out!(0, "", "Enabling module {}", name.magenta());
        module.add_sources(self);
        module.create_files(self, name);
        module.create_hard_links(self, name);
        module.create_symlinks(self, name);
    }

    pub fn disable_module(&mut self, name: &str) {
        if let Some((module, paths)) = self.module_paths.remove_entry(name) {
            self.disable_module_inner(module, paths);
        } else {
            out!(0, "", "Module {} isn't enabled", name.magenta());
        }
    }

    pub fn disable_all_modules(&mut self) {
        if self.module_paths.is_empty() {
            out!(0, "", "There are no enabled modules");
        } else {
            for (module, paths) in mem::take(&mut self.module_paths) {
                self.disable_module_inner(module, paths);
            }
        }
    }

    fn disable_module_inner(&mut self, name: String, paths: Vec<(PathBuf, PathInfo)>) {
        out!(0, "", "Disabling module {}", name.as_str().magenta());
        out!(1, "", "Removing owned paths");
        // Any paths that can't be removed will be put back into the state.
        let mut unremovable_paths = Vec::new();

        // Paths are removed in reverse order to make sure directories are
        // removed last.
        for (path, info) in paths.into_iter().rev() {
            if info.remove_if_owned(&path) {
                self.paths.remove(&path);
            } else {
                unremovable_paths.push((path, info));
            }
        }

        if !unremovable_paths.is_empty() {
            unremovable_paths.reverse();
            self.module_paths.insert(name, unremovable_paths);
        }
    }

    pub fn update_module(&mut self, name: &str) {
        if let Some((name_rc, paths)) = self.module_paths.remove_entry(name) {
            self.disable_module_inner(name_rc, paths);
            if let Some(module) = config::module(name) {
                self.enable_module_inner(name, module);
            }
        } else {
            out!(0, "", "Module {} isn't enabled", name.magenta());
        }
    }

    pub fn update_all_modules(&mut self) {
        if self.module_paths.is_empty() {
            out!(0, "", "There are no enabled modules");
        } else {
            for (name, paths) in mem::take(&mut self.module_paths) {
                let module = config::module(&name).map(|module| (name.to_string(), module));
                self.disable_module_inner(name, paths);
                if let Some((name, module)) = module {
                    self.enable_module_inner(&name, module);
                }
            }
        }
    }
}

pub enum ModuleFilter {
    All,
    Enabled,
    Disabled,
}
