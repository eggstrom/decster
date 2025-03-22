use std::{collections::HashSet, fs, path::Path};

use anyhow::Result;
use tempfile::TempDir;

use crate::{paths, source::Source, utils};

pub struct State {
    enabled: HashSet<String>,
}

impl State {
    pub fn new() -> Result<Self> {
        let enabled = fs::read_to_string(paths::data()?.join("enabled.toml"))
            .ok()
            .map(|s| toml::from_str(&s))
            .transpose()?
            .unwrap_or(HashSet::new());
        Ok(State { enabled })
    }

    pub fn save(&self) -> Result<()> {
        fs::write(
            paths::data()?.join("enabled.toml"),
            toml::to_string(&self.enabled)?,
        )?;
        Ok(())
    }

    pub fn source_builder(&self) -> Result<SourceBuilder> {
        Ok(SourceBuilder {
            dir: TempDir::new()?,
        })
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
