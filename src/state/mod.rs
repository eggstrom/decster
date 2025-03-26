use std::{
    collections::{HashMap, VecDeque},
    fs, io, mem,
    path::Path,
    rc::Rc,
};

use anyhow::{Context, Result};
use crossterm::style::{Attribute, Stylize};
use path_info::PathInfo;
use serde::{Deserialize, Serialize};

use crate::{
    paths,
    source::{Source, name::SourceName},
    utils::{self, output::Pretty},
};

pub mod path_info;

#[derive(Default, Deserialize, Serialize)]
pub struct State {
    module_paths: HashMap<Rc<str>, Vec<Rc<Path>>>,
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

    /// Gets the owner of `path`.
    pub fn owner<P>(&self, path: P) -> Option<&str>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        self.path_info.get(path).map(|(module, _)| module.as_ref())
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
            self.add(module, path, PathInfo::new_dir())
        }
        Ok(())
    }

    pub fn add_file(&mut self, module: &str, path: &Path) {
        match PathInfo::new_file(path) {
            Ok(info) => self.add(module, path, info),
            Err(err) => println!(
                "      {} Couldn't add file to state ({err})",
                "Error:".red()
            ),
        }
    }

    pub fn add_hard_link(&mut self, module: &str, link: &Path) {
        match PathInfo::new_hard_link(link) {
            Ok(info) => self.add(module, link, info),
            Err(err) => println!(
                "      {} Couldn't add hard link to state ({err})",
                "Error:".red()
            ),
        }
    }

    pub fn add_symlink(&mut self, module: &str, original: &Path, link: &Path) {
        self.add(module, link, PathInfo::new_symlink(original));
    }

    pub fn disable_module(&mut self, module: &str) {
        if let Some((module, paths)) = self.module_paths.remove_entry(module) {
            self.disable_module_inner(module, paths);
        } else {
            println!("Module {} isn't enabled", module.magenta());
        }
    }

    pub fn disable_all_modules(&mut self) {
        if self.module_paths.is_empty() {
            println!("There are no disabled modules");
        } else {
            for (module, paths) in mem::take(&mut self.module_paths) {
                self.disable_module_inner(module, paths);
            }
        }
    }

    fn disable_module_inner(&mut self, module: Rc<str>, paths: Vec<Rc<Path>>) {
        println!("Disabling module {}", module.magenta());
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
            self.module_paths.insert(module, unremovable_paths.into());
        }
    }
}
