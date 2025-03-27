use std::{
    collections::{BTreeMap, HashSet},
    fs, io, mem,
    path::Path,
    rc::Rc,
};

use anyhow::{Context, Result};
use crossterm::style::Stylize;
use path_info::PathInfo;
use serde::{Deserialize, Serialize};

use crate::{
    global::{config, paths},
    module::Module,
    out,
    source::{Source, name::SourceName},
    utils::{self, output::Pretty},
};

pub mod path_info;

#[derive(Default, Deserialize, Serialize)]
pub struct State {
    module_paths: BTreeMap<Rc<str>, Vec<(Rc<Path>, PathInfo)>>,
    paths: HashSet<Rc<Path>>,
}

impl State {
    pub fn load() -> Result<Self> {
        fs::create_dir_all(paths::sources())
            .with_context(|| format!("Couldn't create path: {}", paths::sources().pretty()))?;

        Ok(fs::read_to_string(paths::state())
            .ok()
            .and_then(|string| toml::from_str(&string).ok())
            .unwrap_or_default())
    }

    pub fn save(&self) -> Result<()> {
        let path = paths::state();
        fs::write(path, toml::to_string(self)?)?;
        Ok(())
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
        let path = Rc::from(path);
        self.paths.insert(Rc::clone(&path));
        if let Some(paths) = self.module_paths.get_mut(module) {
            paths.push((path, info));
        } else {
            self.module_paths.insert(module.into(), vec![(path, info)]);
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
    ) -> impl Iterator<Item = (&str, &Module, Option<&Vec<(Rc<Path>, PathInfo)>>)> {
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
            out!("Module {} isn't disabled", name.magenta());
        } else if let Some(module) = config::module(name) {
            self.enable_module_inner(name, module);
        } else {
            out!("Module {} isn't defined", name.magenta());
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
            out!("There are no disabled modules");
        }
    }

    fn enable_module_inner(&mut self, name: &str, module: &Module) {
        out!("Enabling module {}", name.magenta());
        module.add_sources(self);
        module.create_files(self, name);
        module.create_hard_links(self, name);
        module.create_symlinks(self, name);
    }

    pub fn disable_module(&mut self, name: &str) {
        if let Some((module, paths)) = self.module_paths.remove_entry(name) {
            self.disable_module_inner(module, paths);
        } else {
            out!("Module {} isn't enabled", name.magenta());
        }
    }

    pub fn disable_all_modules(&mut self) {
        if self.module_paths.is_empty() {
            out!("There are no enabled modules");
        } else {
            for (module, paths) in mem::take(&mut self.module_paths) {
                self.disable_module_inner(module, paths);
            }
        }
    }

    fn disable_module_inner(&mut self, name: Rc<str>, paths: Vec<(Rc<Path>, PathInfo)>) {
        out!("Disabling module {}", name.magenta());
        out!("  Removing owned paths");
        // Any paths that can't be removed will be put back into the state.
        let mut unremovable_paths = Vec::new();

        // Paths are removed in reverse order to make sure directories are
        // removed last.
        for (path, info) in paths.into_iter().rev() {
            if let Err(err) = info.remove_if_owned(&path) {
                out!("{} {} ({err})", "Failed:".red(), path.pretty());
                unremovable_paths.push((path, info));
            } else {
                self.paths.remove(&path);
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
            out!("Module {} isn't enabled", name.magenta());
        }
    }

    pub fn update_all_modules(&mut self) {
        if self.module_paths.is_empty() {
            out!("There are no enabled modules");
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
