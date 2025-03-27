use std::{
    collections::{BTreeMap, HashMap, VecDeque},
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
    source::{Source, name::SourceName},
    utils::{self, fs::Sha256Hash, output::Pretty},
};

pub mod path_info;

#[derive(Default, Deserialize, Serialize)]
pub struct State {
    module_paths: BTreeMap<Rc<str>, Vec<Rc<Path>>>,
    path_info: HashMap<Rc<Path>, (Rc<str>, PathInfo)>,
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
        let path = path.as_ref();
        self.path_info.get(path).is_some()
    }

    pub fn add_source(&self, name: &SourceName, source: &Source) -> Result<()> {
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

    fn add(&mut self, module: &str, path: &Path, info: PathInfo) {
        let module = Rc::from(module);
        let path = Rc::from(path);
        self.module_paths
            .entry(Rc::clone(&module))
            .or_insert_with(|| Vec::new())
            .push(Rc::clone(&path));
        self.path_info.insert(path, (module, info));
    }

    pub fn create_dir(&mut self, module: &str, path: &Path) -> io::Result<()> {
        if !path.is_dir() {
            fs::create_dir(path)?;
            self.add(module, path, PathInfo::Directory);
        }
        Ok(())
    }

    pub fn add_file(&mut self, module: &str, path: &Path, size: u64, hash: Sha256Hash) {
        self.add(module, path, PathInfo::File { size, hash });
    }

    pub fn add_hard_link(&mut self, module: &str, path: &Path, size: u64, hash: Sha256Hash) {
        self.add(module, path, PathInfo::HardLink { size, hash });
    }

    pub fn add_symlink(&mut self, module: &str, original: &Path, link: &Path) {
        let path = original.to_path_buf();
        self.add(module, link, PathInfo::Symlink { path });
    }

    pub fn enable_module(&mut self, name: &str) {
        if self.is_module_enabled(name) {
            println!("Module {} is already enabled", name.magenta());
        } else if let Some(module) = config::module(name) {
            self.enable_module_inner(name, module);
        } else {
            println!("Module {} isn't defined", name.magenta());
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
            println!("There are no disabled modules");
        }
    }

    fn enable_module_inner(&mut self, name: &str, module: &Module) {
        println!("Enabling module {}", name.magenta());
        module.add_sources(self);
        module.create_files(self, name);
        module.create_hard_links(self, name);
        module.create_symlinks(self, name);
    }

    pub fn disable_module(&mut self, name: &str) {
        if let Some((module, paths)) = self.module_paths.remove_entry(name) {
            self.disable_module_inner(module, paths);
        } else {
            println!("Module {} isn't enabled", name.magenta());
        }
    }

    pub fn disable_all_modules(&mut self) {
        if self.module_paths.is_empty() {
            println!("There are no enabled modules");
        } else {
            for (module, paths) in mem::take(&mut self.module_paths) {
                self.disable_module_inner(module, paths);
            }
        }
    }

    fn disable_module_inner(&mut self, name: Rc<str>, paths: Vec<Rc<Path>>) {
        println!("Disabling module {}", name.magenta());
        // Any paths that can't be removed will be put back into the state.
        // This is a VecDeque because they need to be insrted in reverse order.
        let mut unremovable_paths = VecDeque::new();

        // Paths are removed in reverse order to make sure directories are
        // removed last.
        for path in paths.into_iter().rev() {
            if let Some((_, path_info)) = self.path_info.get(&path) {
                if let Err(err) = path_info.remove_if_owned(&path) {
                    println!("{} {} ({err})", "Failed:".red(), path.pretty());
                    unremovable_paths.push_front(path);
                } else {
                    self.path_info.remove(&path);
                }
            }
        }

        if !unremovable_paths.is_empty() {
            self.module_paths.insert(name, unremovable_paths.into());
        }
    }

    pub fn update_module(&mut self, name: &str) {
        if let Some((name_rc, paths)) = self.module_paths.remove_entry(name) {
            self.disable_module_inner(name_rc, paths);
            if let Some(module) = config::module(name) {
                self.enable_module_inner(name, module);
            }
        } else {
            println!("Module {} isn't enabled", name.magenta());
        }
    }

    pub fn update_all_modules(&mut self) {
        if self.module_paths.is_empty() {
            println!("There are no enabled modules");
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
