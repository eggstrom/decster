use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};

pub struct Paths {
    home_dir: PathBuf,
    config_file: PathBuf,
    module_dir: PathBuf,
    static_source_dir: PathBuf,
    dynamic_source_file: PathBuf,
    named_source_dir: PathBuf,
    unnamed_source_dir: PathBuf,
    state_file: PathBuf,
}

impl Paths {
    const APP_NAME: &str = "decster";

    pub fn load(config_dir: Option<PathBuf>) -> Result<Self> {
        let home_dir =
            dirs::home_dir().ok_or(anyhow!("Couldn't determine path of home directory"))?;
        let config_dir = config_dir
            .or(dirs::config_dir().map(|path| path.join(Self::APP_NAME)))
            .ok_or(anyhow!("Couldn't determine path of config directory"))?;
        let data_dir = dirs::data_dir()
            .map(|path| path.join(Self::APP_NAME))
            .ok_or(anyhow!("Couldn't determine path of data directory"))?;

        let config_file = config_dir.join("config.toml");
        let module_dir = config_dir.join("modules");
        let static_source_dir = config_dir.join("sources");
        let dynamic_source_file = config_dir.join("sources.toml");

        let named_source_dir = data_dir.join("named-sources");
        let unnamed_source_dir = data_dir.join("unnamed-sources");
        fs::create_dir_all(&named_source_dir)
            .and_then(|()| fs::create_dir_all(&unnamed_source_dir))
            .map_err(|err| anyhow!("Couldn't create data directory ({err})"))?;

        Ok(Paths {
            home_dir,
            config_file,
            module_dir,
            static_source_dir,
            dynamic_source_file,
            named_source_dir,
            unnamed_source_dir,
            state_file: data_dir.join("state"),
        })
    }

    pub fn home_dir(&self) -> &Path {
        &self.home_dir
    }

    pub fn config_file(&self) -> &Path {
        &self.config_file
    }

    pub fn module_dir(&self) -> &Path {
        &self.module_dir
    }

    pub fn static_source_dir(&self) -> &Path {
        &self.static_source_dir
    }

    pub fn dynamic_source_file(&self) -> &Path {
        &self.dynamic_source_file
    }

    pub fn named_source_dir(&self) -> &Path {
        &self.named_source_dir
    }

    pub fn unnamed_source_dir(&self) -> &Path {
        &self.unnamed_source_dir
    }

    pub fn state_file(&self) -> &Path {
        &self.state_file
    }
}
