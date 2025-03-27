use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};

const APP_NAME: &str = "decster";

pub(super) struct Paths {
    pub home: PathBuf,
    pub config: PathBuf,
    pub data: PathBuf,
    pub sources: PathBuf,
    pub state: PathBuf,
}

impl Paths {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().ok_or(anyhow!("Couldn't determine path of home directory"))?;
        let config = dirs::config_dir()
            .map(|path| path.join(APP_NAME))
            .ok_or(anyhow!("Couldn't determine path of config directory"))?;
        let data = dirs::data_dir()
            .map(|path| path.join(APP_NAME))
            .ok_or(anyhow!("Couldn't determine path of data directory"))?;
        let sources = data.join("sources");
        let state = data.join("state.toml");

        Ok(Paths {
            home,
            config,
            data,
            sources,
            state,
        })
    }
}

fn paths() -> &'static Paths {
    &super::state().paths
}

pub fn home() -> &'static Path {
    &paths().home
}

pub fn config() -> &'static Path {
    &paths().config
}

pub fn data() -> &'static Path {
    &paths().data
}

pub fn sources() -> &'static Path {
    &paths().sources
}

pub fn state() -> &'static Path {
    &paths().state
}
