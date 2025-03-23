use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    fs::{self, File},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use anyhow::Result;
use bincode::{Decode, Encode, config::Configuration};
use tempfile::TempDir;

use crate::{
    link::LinkMethod,
    paths,
    source::Source,
    utils::{self, Sha256Hash},
};

#[derive(Decode, Default, Encode)]
pub struct State {
    owned_files: HashMap<PathBuf, FileInfo>,
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

    /// Checks whether `path` can be written to due to not existing, being
    /// an empty directory, or being owned by the program.
    pub fn check<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        path.metadata()
            .is_ok_and(|metadata| !metadata.permissions().readonly())
            || (!path.exists())
            || (path.read_dir().is_ok_and(|mut dir| dir.next().is_none()))
            || self
                .owned_files
                .get(path)
                .map(|file| file.check(path))
                .is_some_and(|owned| owned)
    }

    pub fn add_file<P>(&mut self, path: P, method: LinkMethod) -> Result<()>
    where
        P: Into<PathBuf>,
    {
        let path = path.into();
        let file = match method {
            LinkMethod::Copy | LinkMethod::HardLink => FileInfo::file_at(&path)?,
            LinkMethod::SoftLink => FileInfo::link_at(&path)?,
        };
        self.owned_files.insert(path, file);
        Ok(())
    }

    pub fn remove_file<P>(&mut self, path: P)
    where
        P: AsRef<Path>,
    {
        self.owned_files.remove(path.as_ref());
    }

    pub fn source_builder(&self) -> Result<SourceBuilder> {
        Ok(SourceBuilder {
            dir: TempDir::new()?,
        })
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for path in self.owned_files.keys() {
            writeln!(f, "{}", path.display())?;
        }
        Ok(())
    }
}

#[derive(Decode, Encode)]
enum FileInfo {
    File { size: u64, hash: Sha256Hash },
    Link { path: PathBuf },
}

impl FileInfo {
    pub fn file_at<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        Ok(FileInfo::File {
            size: path.metadata()?.size(),
            hash: utils::hash_file(path)?,
        })
    }

    pub fn link_at<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        Ok(FileInfo::Link {
            path: path.read_link()?,
        })
    }

    /// Checks whether the contents of `path` match `self`.
    pub fn check<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        match self {
            FileInfo::File { size, hash } => {
                path.metadata()
                    .is_ok_and(|metadata| metadata.size() == *size)
                    && utils::hash_file(path).is_ok_and(|hash2| hash2 == *hash)
            }
            FileInfo::Link { path } => path.read_link().is_ok_and(|path2| path2 == *path),
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
