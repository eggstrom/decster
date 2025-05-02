use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};

pub struct Dirs {
    home: PathBuf,
    config: PathBuf,
    modules: PathBuf,
    static_sources: PathBuf,
    dynamic_sources: PathBuf,
    named_sources: PathBuf,
    unnamed_sources: PathBuf,
    state: PathBuf,
}

impl Dirs {
    const APP_NAME: &str = "decster";

    pub fn load(config_dir: Option<PathBuf>) -> Result<Self> {
        let home = dirs::home_dir().ok_or(anyhow!("Couldn't determine path of home directory"))?;
        let config = config_dir
            .or(dirs::config_dir().map(|path| path.join(Self::APP_NAME)))
            .ok_or(anyhow!("Couldn't determine path of config directory"))?;
        let data = dirs::data_dir()
            .map(|path| path.join(Self::APP_NAME))
            .ok_or(anyhow!("Couldn't determine path of data directory"))?;

        let named_sources = data.join("named-sources");
        let unnamed_sources = data.join("unnamed-sources");
        fs::create_dir_all(&named_sources)
            .and_then(|()| fs::create_dir_all(&unnamed_sources))
            .map_err(|err| anyhow!("Couldn't create data directory ({err})"))?;

        Ok(Dirs {
            home,
            config: config.join("config.toml"),
            modules: config.join("modules"),
            static_sources: config.join("sources"),
            dynamic_sources: config.join("sources.toml"),
            named_sources,
            unnamed_sources,
            state: data.join("state"),
        })
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    pub fn config(&self) -> &Path {
        &self.config
    }

    pub fn modules(&self) -> &Path {
        &self.modules
    }

    pub fn static_sources(&self) -> &Path {
        &self.static_sources
    }

    pub fn dynamic_sources(&self) -> &Path {
        &self.dynamic_sources
    }

    pub fn named_sources(&self) -> &Path {
        &self.named_sources
    }

    pub fn unnamed_sources(&self) -> &Path {
        &self.unnamed_sources
    }

    pub fn state(&self) -> &Path {
        &self.state
    }
}
