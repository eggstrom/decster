use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};

#[derive(Debug)]
pub struct Paths {
    home: PathBuf,
    config: PathBuf,
    data: PathBuf,
}

impl Paths {
    pub fn new(config: Option<PathBuf>) -> Result<Self> {
        let home = dirs::home_dir().ok_or(anyhow!("couldn't find home directory"))?;
        let config = config
            .or_else(|| dirs::config_dir().map(|path| path.join("decster")))
            .ok_or(anyhow!("couldn't find config directory"))?;
        let data = dirs::data_dir()
            .ok_or(anyhow!("couldn't find data directory"))?
            .join("decster");
        Ok(Paths { home, config, data })
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    pub fn config(&self) -> &Path {
        &self.config
    }

    pub fn data(&self) -> &Path {
        &self.data
    }
}
