use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use anyhow::{Result, anyhow};

static HOME: LazyLock<Option<PathBuf>> = LazyLock::new(|| dirs::home_dir());
static CONFIG: LazyLock<Option<PathBuf>> =
    LazyLock::new(|| dirs::config_dir().map(|path| path.join("decster")));
static DATA: LazyLock<Option<PathBuf>> =
    LazyLock::new(|| dirs::data_dir().map(|path| path.join("decster")));
static SOURCES: LazyLock<Option<PathBuf>> =
    LazyLock::new(|| DATA.as_deref().map(|path| path.join("sources")));
static STATE: LazyLock<Option<PathBuf>> =
    LazyLock::new(|| DATA.as_deref().map(|path| path.join("state.toml")));

pub fn home<'a>() -> Result<&'a Path> {
    HOME.as_deref()
        .ok_or(anyhow!("Couldn't determine path of home directory"))
}

pub fn config<'a>() -> Result<&'a Path> {
    CONFIG
        .as_deref()
        .ok_or(anyhow!("Couldn't determine path of config directory"))
}

pub fn data<'a>() -> Result<&'a Path> {
    DATA.as_deref()
        .ok_or(anyhow!("Couldn't determine path of data directory"))
}

pub fn sources<'a>() -> Result<&'a Path> {
    SOURCES
        .as_deref()
        .ok_or(anyhow!("Couldn't determine path of source directory"))
}

pub fn state<'a>() -> Result<&'a Path> {
    STATE
        .as_deref()
        .ok_or(anyhow!("Couldn't determine path of state file"))
}
