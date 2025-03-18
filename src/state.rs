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
        
        if path.exists() {
            if path.is_dir() {
                fs::remove_dir_all(path)?;
            } else {
                fs::remove_file(path)?;
            }
        }
        fs::create_dir_all(path)?;
        fs::rename(self.dir.path(), path)?;
        Ok(self.state)
    }

    pub fn add_source(&self, name: &str, source: &Source) -> Result<()> {
        Ok(match source {
            Source::Path(path) => self.add_path_source(name, path)?,
        })
    }

    fn add_path_source<P>(&self, name: &str, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        println!("Adding source: {} ({})", name, path.display());
        utils::copy_all(path, self.dir.path().join(name))?;
        Ok(())
    }
}
