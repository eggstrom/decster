use std::{
    collections::HashMap,
    fs::{self, File},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use anyhow::Result;
use bincode::{Decode, Encode, config::Configuration};
use tempfile::TempDir;

use crate::{
    paths,
    source::Source,
    utils::{self, Sha256Hash},
};

#[derive(Decode, Default, Encode)]
pub struct State {
    owned_files: HashMap<PathBuf, OwnedFile>,
}

impl State {
    pub fn load() -> Result<Self> {
        Ok(File::open(paths::state()?)
            .ok()
            .and_then(|mut file| bincode::decode_from_std_read(&mut file, Self::bin_config()).ok())
            .unwrap_or_default())
    }

    pub fn save(&self) -> Result<()> {
        let mut file = File::create(paths::state()?)?;
        bincode::encode_into_std_write(self, &mut file, Self::bin_config())?;
        Ok(())
    }

    pub fn bin_config() -> Configuration {
        bincode::config::standard()
    }

    /// Checks whether `path` can be written to due to being empty or owned by
    /// the program.
    pub fn check<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        self.owned_files
            .get(path)
            .is_some_and(|file| file.check(path))
    }

    pub fn add_file(&mut self, path: PathBuf, file: OwnedFile) {
        self.owned_files.insert(path, file);
    }

    pub fn source_builder(&self) -> Result<SourceBuilder> {
        Ok(SourceBuilder {
            dir: TempDir::new()?,
        })
    }
}

#[derive(Decode, Encode)]
enum OwnedFile {
    Copy { size: u64, hash: Sha256Hash },
    HardLink { size: u64, hash: Sha256Hash },
    SoftLink { path: PathBuf },
}

impl OwnedFile {
    pub fn new_file(size: u64, hash: Sha256Hash) -> Self {
        OwnedFile::Copy { size, hash }
    }

    pub fn new_hard_link(size: u64, hash: Sha256Hash) -> Self {
        OwnedFile::HardLink { size, hash }
    }

    pub fn new_soft_link(link_path: PathBuf) -> Self {
        OwnedFile::SoftLink { path: link_path }
    }

    /// Checks whether `path` has no contents or if it's contents are
    /// accurately described by `self`.
    pub fn check<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        match self {
            OwnedFile::Copy { size, hash } | OwnedFile::HardLink { size, hash } => {
                path.metadata()
                    .is_ok_and(|metadata| metadata.size() == *size)
                    && utils::hash_file(path).is_ok_and(|hash2| hash2 == *hash)
            }
            OwnedFile::SoftLink { path: link_path } => {
                path.read_link().is_ok_and(|path| path == *link_path)
            }
        }
    }
}

pub struct SourceBuilder {
    dir: TempDir,
}

impl SourceBuilder {
    pub fn save(self) -> Result<()> {
        let path = paths::sources()?;
        utils::remove_all(&path)?;
        fs::create_dir_all(&path)?;
        fs::rename(self.dir.path(), &path)?;
        Ok(())
    }

    pub fn add_source(&self, name: &str, source: &Source) -> Result<()> {
        Ok(match source {
            Source::Text(text) => self.add_text_source(name, text)?,
            Source::Path(path) => self.add_path_source(name, path)?,
        })
    }

    fn add_text_source(&self, name: &str, text: &str) -> Result<()> {
        let path = self.dir.path().join(name);
        println!("Adding source: {} (text)", name);

        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        }
        fs::write(path, text)?;
        Ok(())
    }

    fn add_path_source<P>(&self, name: &str, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        println!("Adding source: {} (path: {})", name, path.display());

        utils::copy_all(path, self.dir.path().join(name))?;
        Ok(())
    }
}
