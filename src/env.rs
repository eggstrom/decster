use std::{
    borrow::Cow,
    fs,
    os::unix,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::{Context, Result, anyhow};
use nix::unistd;

use crate::utils::pretty::Pretty;

#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct User {
    uid: u32,
    home: PathBuf,
}

impl User {
    pub fn new(name: &str) -> Result<Self> {
        unistd::User::from_name(name)?
            .map(|user| User {
                uid: user.uid.as_raw(),
                home: user.dir,
            })
            .ok_or(anyhow!("User doesn't exist"))
    }

    pub fn is_current(&self) -> bool {
        self.uid == uid()
    }

    pub fn tildefy<'a>(&self, path: &'a Path) -> Cow<'a, Path> {
        tildefy_with(path, &self.home)
    }

    pub fn untildefy<'a>(&self, path: &'a Path) -> Cow<'a, Path> {
        untildefy_with(path, &self.home)
    }

    pub fn change_owner(&self, path: &Path) -> Result<()> {
        unix::fs::lchown(path, Some(self.uid), None)
            .with_context(|| format!("Couldn't change owner of {}", tildefy(path).pretty()))
    }
}

struct Env {
    uid: u32,
    home: PathBuf,
    config: PathBuf,
    modules: PathBuf,
    config_sources: PathBuf,
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
            home,
            config: config.join("config.toml"),
            modules: config.join("modules"),
            config_sources: config.join("sources"),
            named_sources,
            unnamed_sources,
            state: data.join("state"),
        })
    }
}

static ENV: OnceLock<Env> = OnceLock::new();

pub fn load(config_dir: Option<PathBuf>) -> Result<()> {
    ENV.set(Env::load(config_dir)?)
        .ok()
        .expect("`env::load` should only be called once");
    Ok(())
}

fn env() -> &'static Env {
    ENV.get().expect(
        "`env::load` should be called without failing before other functions in `env` are called",
    )
}

fn uid() -> u32 {
    env().uid
}

pub fn home() -> &'static Path {
    &env().home
}

pub fn config() -> &'static Path {
    &env().config
}

pub fn modules() -> &'static Path {
    &env().modules
}

pub fn config_sources() -> &'static Path {
    &env().config_sources
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

fn tildefy_with<'a>(path: &'a Path, home: &Path) -> Cow<'a, Path> {
    match path.strip_prefix(home) {
        Ok(path) => match path.parent() {
            None => Cow::Borrowed(Path::new(TILDE)),
            Some(_) => Cow::Owned(Path::new(TILDE).join(path)),
        },
        Err(_) => Cow::Borrowed(path),
    }
}

fn untildefy_with<'a>(path: &'a Path, home: &Path) -> Cow<'a, Path> {
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
