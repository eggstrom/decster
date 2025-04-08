use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File},
    os::unix,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail, ensure};
use bincode::{Decode, Encode, config::Configuration};
use globset::GlobSet;
use path::PathInfo;

use crate::{
    config,
    module::Module,
    out, paths,
    source::{ident::SourceIdent, info::SourceInfo},
    users::Users,
    utils::{glob::GlobSetExt, output::PathDisplay},
};

pub mod path;

#[derive(Decode, Default, Encode)]
pub struct State {
    sources: BTreeMap<SourceIdent, SourceInfo>,
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

    pub fn has_source(&self, ident: &SourceIdent, source: &SourceInfo) -> bool {
        self.sources.get(ident).is_some_and(|s| s == source) && ident.path().exists()
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

    pub fn add_source(&mut self, ident: &SourceIdent, source: &SourceInfo) {
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

    fn modules_matching_globs(&self, globs: &[String]) -> Result<Vec<String>> {
        let globs = GlobSet::from_globs(globs)?;
        let matches: Vec<_> = self
            .module_paths
            .keys()
            .filter(move |name| globs.is_match(name))
            .cloned()
            .collect();
        ensure!(
            !matches.is_empty(),
            "Patterns didn't match any enabled modules"
        );
        Ok(matches)
    }

    pub fn enable_module(&mut self, users: &mut Users, name: &str, module: &Module) {
        out!(0; "Enabling module {}", name.magenta());
        module.fetch_sources(self, name);
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

    fn disable_module(&mut self, name: &str) {
        out!(0; "Disabling module {}", name.magenta());
        out!(1; "Removing owned paths");

        let paths = self
            .module_paths
            .get_mut(name)
            .expect("Whether `name` exists should be checked before calling this method");
        // Paths are removed in reverse order to make sure directories are
        // removed last.
        for i in (0..paths.len()).rev() {
            let (path, info) = &paths[i];
            if info.remove_if_owned(path) {
                self.paths.remove(path);
                paths.remove(i);
            }
        }
        if paths.is_empty() {
            self.module_paths.remove(name);
        }
    }

    pub fn disable_modules_matching_globs(&mut self, globs: &[String]) -> Result<()> {
        for name in self.modules_matching_globs(globs)? {
            self.disable_module(&name);
        }
        Ok(())
    }

    fn can_update(&self) -> Result<()> {
        if self.module_paths.is_empty() {
            bail!("There are no enabled modules to update");
        }
        Ok(())
    }

    fn update_module(&mut self, users: &mut Users, name: &str, module: Option<&Module>) {
        self.disable_module(name);
        if let Some(module) = module {
            self.enable_module(users, name, module);
        }
    }

    pub fn update_all_modules(&mut self, users: &mut Users) -> Result<()> {
        self.can_update()?;
        for name in self.module_paths.keys().cloned().collect::<Vec<_>>() {
            self.update_module(users, &name, config::module(&name));
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
            self.update_module(users, &name, config::module(&name));
        }
        Ok(())
    }
}

pub enum ModuleFilter {
    All,
    Enabled,
    Disabled,
}
