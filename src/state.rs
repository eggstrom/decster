use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    rc::Rc,
};

use anyhow::{Context, Result, anyhow};
use crossterm::style::Stylize;
use log::info;
use serde::{Deserialize, Serialize};

use crate::{
    paths,
    source::{Source, SourceName},
    utils::{self, Sha256Hash},
};

#[derive(Default, Deserialize, Serialize)]
pub struct State {
    module_paths: HashMap<Rc<str>, Vec<Rc<Path>>>,
    path_info: HashMap<Rc<Path>, (Rc<str>, PathInfo)>,
}

impl State {
    pub fn load() -> Result<Self> {
        let source_path = paths::sources()?;
        fs::create_dir_all(paths::sources()?)
            .with_context(|| format!("Couldn't create path: {}", source_path.display()))?;

        Ok(fs::read_to_string(paths::state()?)
            .ok()
            .and_then(|string| toml::from_str(&string).ok())
            .unwrap_or_default())
    }

    pub fn save(&self) -> Result<()> {
        let path = paths::state()?;
        fs::write(path, toml::to_string(self)?)
            .with_context(|| format!("Couldn't write to file: {}", path.display()))?;
        Ok(())
    }

    /// Gets the owner of `path`.
    pub fn owner<P>(&self, path: P) -> Option<&str>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        self.path_info.get(path).map(|(module, _)| module.as_ref())
    }

    /// Checks whether `path` is owned and if it's contents match what they're
    /// expected to have.
    fn is_owned<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        self.path_info
            .get(path)
            .map(|(_, info)| info.matches(path))
            .is_some_and(|owned| owned)
    }

    pub fn is_writable<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        (!path.exists())
            || (path.read_dir().is_ok_and(|mut dir| dir.next().is_none()))
            || self.is_owned(path)
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

    pub fn add_dir(&mut self, module: &str, path: &Path) {
        self.add(module, path, PathInfo::Directory);
    }

    pub fn add_file(&mut self, module: &str, path: &Path) -> Result<()> {
        self.add(module, path, PathInfo::new_file(path)?);
        Ok(())
    }

    pub fn add_link(&mut self, module: &str, path: &Path) -> Result<()> {
        self.add(module, path, PathInfo::new_link(path)?);
        Ok(())
    }

    pub fn remove_module(&mut self, name: &str) -> Result<()> {
        // Paths are removed in reverse order to make sure directories are
        // removed last.
        for path in self
            .module_paths
            .remove(name)
            .ok_or(anyhow!("Couldn't find module: {}", name.magenta()))?
            .into_iter()
            .rev()
        {
            if let Some((_, path_info)) = self.path_info.remove(&path) {
                path_info
                    .remove_if_matches(&path)
                    .with_context(|| format!("Couldn't remove: {}", path.display()))?;
            }
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
        info!("Adding text source: {}", name.magenta());

        let source_path = paths::sources()?.join(name);
        fs::write(&source_path, text)
            .with_context(|| format!("Couldn't write to file: {}", source_path.display()))?;
        Ok(())
    }

    fn add_path_source<P>(&self, name: &SourceName, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        info!("Adding path source: {}", name.magenta());

        let source_path = paths::sources()?.join(name);
        utils::remove_all(&source_path)?;
        utils::copy_all(path, &source_path)?;
        Ok(())
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (module, paths) in self.module_paths.iter() {
            writeln!(f, "{}", module.magenta())?;
            for path in paths.iter() {
                writeln!(f, "  {}", path.display())?;
            }
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum PathInfo {
    Directory,
    File { size: u64, hash: Sha256Hash },
    Link { path: PathBuf },
}

impl PathInfo {
    pub fn new_link<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        Ok(PathInfo::Link {
            path: path
                .read_link()
                .with_context(|| format!("Couldn't read symlink: {}", path.display()))?,
        })
    }

    pub fn new_file<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        Ok(PathInfo::File {
            size: path
                .metadata()
                .with_context(|| format!("Couldn't read file metadata: {}", path.display()))?
                .size(),
            hash: utils::hash_file(path)?,
        })
    }

    /// Checks whether the contents of `path` match `self`.
    pub fn matches<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        match self {
            PathInfo::Directory => path.is_dir(),
            PathInfo::File { size, hash } => {
                path.metadata()
                    .is_ok_and(|metadata| metadata.size() == *size)
                    && utils::hash_file(path).is_ok_and(|hash2| hash2 == *hash)
            }
            PathInfo::Link { path } => path.read_link().is_ok_and(|path2| path2 == *path),
        }
    }

    /// Removes `path` if it's contents match `self`.
    pub fn remove_if_matches<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        if self.matches(path) {
            if path.is_dir() {
                fs::remove_dir(path)?;
            } else {
                fs::remove_file(path)?;
            }
        }
        Ok(())
    }
}
