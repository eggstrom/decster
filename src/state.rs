use std::{fs, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

use crate::{source::Source, utils};

#[derive(Default, Deserialize, Serialize)]
pub struct State {}

impl State {
    pub fn builder() -> Result<StateBuilder> {
        Ok(StateBuilder {
            dir: TempDir::new()?,
            state: State::default(),
        })
    }
}

pub struct StateBuilder {
    dir: TempDir,
    state: State,
}

impl StateBuilder {
    pub fn build<P>(self, path: P) -> Result<State>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        utils::remove_all(path)?;
        fs::create_dir_all(path)?;
        fs::rename(self.dir.path(), path)?;
        Ok(self.state)
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
