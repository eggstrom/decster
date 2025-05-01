use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use nix::unistd;

pub(super) struct Env {
    pub uid: u32,
    pub gid: u32,

    pub home: PathBuf,
    pub config: PathBuf,
    pub modules: PathBuf,
    pub static_sources: PathBuf,
    pub dynamic_sources: PathBuf,
    pub named_sources: PathBuf,
    pub unnamed_sources: PathBuf,
    pub state: PathBuf,
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
}

fn env() -> &'static Env {
    &super::state().env
}

pub fn uid() -> u32 {
    env().uid
}

pub fn gid() -> u32 {
    env().gid
}

pub fn home() -> &'static Path {
    &env().home
}

pub fn static_sources() -> &'static Path {
    &env().static_sources
}

pub fn named_sources() -> &'static Path {
    &env().named_sources
}

pub fn unnamed_sources() -> &'static Path {
    &env().unnamed_sources
}

pub fn state() -> &'static Path {
    &env().state
}

const TILDE: &str = "~";

pub fn tildefy_with<'a>(path: &'a Path, home: &Path) -> Cow<'a, Path> {
    match path.strip_prefix(home) {
        Ok(path) => match path.parent() {
            None => Cow::Borrowed(Path::new(TILDE)),
            Some(_) => Cow::Owned(Path::new(TILDE).join(path)),
        },
        Err(_) => Cow::Borrowed(path),
    }
}

pub fn untildefy_with<'a>(path: &'a Path, home: &Path) -> Cow<'a, Path> {
    match path.strip_prefix(TILDE) {
        Ok(path) => Cow::Owned(home.join(path)),
        Err(_) => Cow::Borrowed(path),
    }
}

pub fn tildefy(path: &Path) -> Cow<Path> {
    tildefy_with(path, home())
}

pub fn untildefy(path: &Path) -> Cow<Path> {
    untildefy_with(path, home())
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
            assert_eq!(tildefy_with(&untilde, &home), tilde);
        }
    }

    #[test]
    fn untildefy() {
        let home = dirs::home_dir().unwrap();
        for (tilde, no_tilde) in paths(&home) {
            assert_eq!(untildefy_with(tilde, &home), no_tilde);
        }
    }
}
