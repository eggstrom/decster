use std::{
    collections::{HashMap, VecDeque},
    fs, io,
    path::Path,
    rc::Rc,
};

use anyhow::{Context, Result, anyhow};
use crossterm::style::Stylize;
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
        let source_path = paths::sources()?;
        fs::create_dir_all(paths::sources()?)
            .with_context(|| format!("Couldn't create path: {}", source_path.pretty()))?;

        Ok(fs::read_to_string(paths::state()?)
            .ok()
            .and_then(|string| toml::from_str(&string).ok())
            .unwrap_or_default())
    }

    pub fn save(&self) -> Result<()> {
        let path = paths::state()?;
        fs::write(path, toml::to_string(self)?)
            .with_context(|| format!("Couldn't write to file: {}", path.pretty()))?;
        Ok(())
    }

    pub fn is_module_enabled(&self, name: &str) -> bool {
        self.module_paths.get(name).is_some()
    }

    /// Gets the owner of `path`.
    pub fn owner<P>(&self, path: P) -> Option<&str>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        self.path_info.get(path).map(|(module, _)| module.as_ref())
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
            Err(error) => println!(
                "      {} Couldn't add file to state ({error})",
                "Error:".red()
            ),
        }
    }

    pub fn add_hard_link(&mut self, module: &str, link: &Path) {
        match PathInfo::new_hard_link(link) {
            Ok(info) => self.add(module, link, info),
            Err(error) => println!(
                "      {} Couldn't add hard link to state ({error})",
                "Error:".red()
            ),
        }
    }

    pub fn add_symlink(&mut self, module: &str, original: &Path, link: &Path) {
        self.add(module, link, PathInfo::new_symlink(original));
    }

    pub fn remove_module(&mut self, name: &str) -> Result<()> {
        // Any paths that can't be removed will be put back into the state.
        // This is a VecDeque because they need to be insrted in reverse order.
        let mut unremovable_paths = VecDeque::new();

        // Paths are removed in reverse order to make sure directories are
        // removed last.
        for path in self
            .module_paths
            .remove(name)
            .ok_or(anyhow!("Couldn't find module: {}", name.magenta()))?
            .into_iter()
            .rev()
        {
            if let Some((_, path_info)) = self.path_info.get(&path) {
                if let Err(error) = path_info.remove_if_owned(&path) {
                    println!("{} {} ({error})", "Failed:".red(), path.pretty());
                    unremovable_paths.push_front(path);
                } else {
                    self.path_info.remove(&path);
                }
            }
        }
        if !unremovable_paths.is_empty() {
            self.module_paths
                .insert(Rc::from(name), unremovable_paths.into());
        }
        Ok(())
    }

    pub fn add_source(&self, name: &SourceName, source: &Source) -> Result<()> {
        match source {
            Source::Text(text) => self.add_text_source(name, text),
            Source::Path(path) => self.add_path_source(name, path),
        }
        .with_context(|| format!("Couldn't add source: {}", name.magenta()))
    }

    fn add_text_source(&self, name: &SourceName, text: &str) -> Result<()> {
        println!("{} {} (text)", "  Added:".green(), name.magenta());

        let source_path = paths::sources()?.join(name);
        fs::write(&source_path, text)
            .with_context(|| format!("Couldn't write to file: {}", source_path.pretty()))?;
        Ok(())
    }

    fn add_path_source<P>(&self, name: &SourceName, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        println!("{} {} (path)", "  Added:".green(), name.magenta());

        let source_path = paths::sources()?.join(name);
        utils::fs::remove_all(&source_path)?;
        utils::fs::copy_all(path, &source_path)?;
        Ok(())
    }
}
