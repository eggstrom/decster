use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use nix::unistd;

pub struct Env {
    uid: u32,
    gid: u32,

    home: PathBuf,
    config: PathBuf,
    modules: PathBuf,
    static_sources: PathBuf,
    dynamic_sources: PathBuf,
    named_sources: PathBuf,
    unnamed_sources: PathBuf,
    state: PathBuf,
}

impl Env {
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

        Ok(Env {
            uid: unistd::getuid().as_raw(),
            gid: unistd::getgid().as_raw(),

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

    pub fn uid(&self) -> u32 {
        self.uid
    }

    pub fn gid(&self) -> u32 {
        self.gid
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

    const TILDE: &str = "~";

    pub fn tildefy_with<'a>(path: &'a Path, home: &Path) -> Cow<'a, Path> {
        match path.strip_prefix(home) {
            Ok(path) => match path.parent() {
                None => Cow::Borrowed(Path::new(Self::TILDE)),
                Some(_) => Cow::Owned(Path::new(Self::TILDE).join(path)),
            },
            Err(_) => Cow::Borrowed(path),
        }
    }

    pub fn untildefy_with<'a>(path: &'a Path, home: &Path) -> Cow<'a, Path> {
        match path.strip_prefix(Self::TILDE) {
            Ok(path) => Cow::Owned(home.join(path)),
            Err(_) => Cow::Borrowed(path),
        }
    }

    pub fn tildefy<'a>(&self, path: &'a Path) -> Cow<'a, Path> {
        Self::tildefy_with(path, &self.home)
    }

    pub fn untildefy<'a>(&self, path: &'a Path) -> Cow<'a, Path> {
        Self::untildefy_with(path, &self.home)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(home: &Path) -> Vec<(&Path, PathBuf)> {
        let home = home.to_string_lossy();
        [
            ("~", format!("{home}")),
            ("~/", format!("{home}/")),
            ("~/foo", format!("{home}/foo")),
            (" ~/foo", " ~/foo".to_string()),
            ("~ /foo", "~ /foo".to_string()),
            ("~bar/foo", "~bar/foo".to_string()),
        ]
        .map(|(tilde, no_tilde)| (Path::new(tilde), PathBuf::from(no_tilde)))
        .into()
    }

    #[test]
    fn tildefy() {
        let home = dirs::home_dir().unwrap();
        for (tilde, untilde) in paths(&home) {
            assert_eq!(Env::tildefy_with(&untilde, &home), tilde);
        }
    }

    #[test]
    fn untildefy() {
        let home = dirs::home_dir().unwrap();
        for (tilde, no_tilde) in paths(&home) {
            assert_eq!(Env::untildefy_with(tilde, &home), no_tilde);
        }
    }
}
