use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::{Result, anyhow};

const APP_NAME: &str = "decster";

struct Paths {
    home: PathBuf,
    config: PathBuf,
    data: PathBuf,
    sources: PathBuf,
    state: PathBuf,
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

static PATHS: OnceLock<Paths> = OnceLock::new();

pub fn init() -> Result<()> {
    PATHS
        .set(Paths::new()?)
        .ok()
        .expect("`paths::init` should only be called once");
    Ok(())
}

fn paths() -> &'static Paths {
    PATHS.get().expect("`paths::init` should be called and return `Ok` before any other function in `paths` is called")
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
