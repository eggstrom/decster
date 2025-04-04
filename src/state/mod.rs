use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File},
    mem,
    os::unix,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use bincode::{Decode, Encode, config::Configuration};
use path::PathInfo;

use crate::{
    config,
    module::Module,
    out, paths,
    source::{definition::SourceDefinition, name::SourceName},
    users::Users,
    utils::output::PathDisplay,
};

pub mod path;

#[derive(Decode, Default, Encode)]
pub struct State {
    sources: BTreeMap<SourceName, SourceDefinition>,
    module_paths: BTreeMap<String, Vec<(PathBuf, PathInfo)>>,
    paths: HashSet<PathBuf>,
}

impl State {
    pub fn load() -> Result<Self> {
        let dir = paths::named_sources();
        fs::create_dir_all(dir)
            .with_context(|| format!("Couldn't create path: {}", dir.display_dir()))?;
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

    pub fn source(&self, name: &SourceName) -> Option<&SourceDefinition> {
        self.sources.get(name)
    }

    pub fn sources(&self) -> impl ExactSizeIterator<Item = (&SourceName, &SourceDefinition)> {
        self.sources.iter()
    }

    pub fn has_source(&self, name: &SourceName, source: &SourceDefinition) -> bool {
        self.sources.get(name).is_some_and(|s| s == source) && name.named_path().exists()
    }

    pub fn has_module(&self, module: &str) -> bool {
        self.module_paths.contains_key(module)
    }

    pub fn has_path<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        self.paths.contains(path.as_ref())
    }

    pub fn add_source(&mut self, name: &SourceName, source: &SourceDefinition) {
        self.sources.insert(name.clone(), source.clone());
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

    pub fn create_dir(&mut self, module: &str, path: &Path) {
        if !path.is_dir() {
            if let Err(err) = fs::create_dir(path) {
                out!(2, R; "{}", path.display_dir(); "{err}");
            } else {
                self.add_path(module, path, PathInfo::Directory);
                out!(2, G; "{}", path.display_dir());
            }
        }
    }

    /// Returns a list of module names, definitions, and owned paths.
    ///
    /// If `modules` isn't empty, all modules not in `modules` will be filtered
    /// out. `filter` determines whether to look for all modules, enabled
    /// modules, or disabled modules.
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

    pub fn enable_module(&mut self, users: &mut Users, name: &str) {
        if self.has_module(name) {
            out!(0, R; "Module {} isn't disabled", name.magenta());
        } else if let Some(module) = config::module(name) {
            self.enable_module_inner(users, name, module);
        } else {
            out!(0, R; "Module {} isn't defined", name.magenta());
        }
    }

    pub fn enable_all_modules(&mut self, users: &mut Users) {
        let mut has_enabled = false;
        for (name, module) in config::modules() {
            if !self.has_module(name) {
                self.enable_module_inner(users, name, module);
                has_enabled = true;
            }
        }
        if !has_enabled {
            out!(0, R; "There are no disabled modules");
        }
    }

    fn enable_module_inner(&mut self, users: &mut Users, name: &str, module: &Module) {
        out!(0; "Enabling module {}", name.magenta());
        module.fetch_sources(self);
        module.create_files(self, name);
        module.create_hard_links(self, name);
        module.create_symlinks(self, name);

        let uid = if let Some(user) = &module.user {
            if users.is_current(user) {
                return;
            }
            out!(1; "Changing file ownership");
            match users.uid(user) {
                Ok(uid) => uid,
                Err(err) => {
                    out!(2, R; "Couldn't get {}'s UID", user.as_str().magenta(); "{err}");
                    return;
                }
            }
        } else {
            return;
        };

        if let Some(module_paths) = self.module_paths.get(name) {
            for (path, info) in module_paths {
                let display = path.display_kind(info.kind());
                match unix::fs::lchown(path, Some(uid.as_raw()), None) {
                    Ok(()) => out!(2, G; "{display}"),
                    Err(err) => out!(2, R; "{display}"; "{err}"),
                }
            }
        }
    }

    pub fn disable_module(&mut self, name: &str) {
        if let Some((module, paths)) = self.module_paths.remove_entry(name) {
            self.disable_module_inner(module, paths);
        } else {
            out!(0, R; "Module {} isn't enabled", name.magenta());
        }
    }

    pub fn disable_all_modules(&mut self) {
        if self.module_paths.is_empty() {
            out!(0, R; "There are no enabled modules");
        } else {
            for (module, paths) in mem::take(&mut self.module_paths) {
                self.disable_module_inner(module, paths);
            }
        }
    }

    fn disable_module_inner(&mut self, name: String, paths: Vec<(PathBuf, PathInfo)>) {
        out!(0; "Disabling module {}", name.as_str().magenta());
        out!(1; "Removing owned paths");
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

    pub fn update_module(&mut self, users: &mut Users, name: &str) {
        if let Some((name_rc, paths)) = self.module_paths.remove_entry(name) {
            self.disable_module_inner(name_rc, paths);
            if let Some(module) = config::module(name) {
                self.enable_module_inner(users, name, module);
            }
        } else {
            out!(0, R; "Module {} isn't enabled", name.magenta());
        }
    }

    pub fn update_all_modules(&mut self, users: &mut Users) {
        if self.module_paths.is_empty() {
            out!(0, R; "There are no enabled modules");
        } else {
            for (name, paths) in mem::take(&mut self.module_paths) {
                let module = config::module(&name).map(|module| (name.to_string(), module));
                self.disable_module_inner(name, paths);
                if let Some((name, module)) = module {
                    self.enable_module_inner(users, &name, module);
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
