use std::{
    borrow::Cow,
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
    fn load(config_dir: Option<PathBuf>) -> Result<Self> {
        let home = dirs::home_dir().ok_or(anyhow!("Couldn't determine path of home directory"))?;
        let config = config_dir
            .or(dirs::config_dir().map(|path| path.join(APP_NAME)))
            .ok_or(anyhow!("Couldn't determine path of config directory"))?;
        let data = dirs::data_dir()
            .map(|path| path.join(APP_NAME))
            .ok_or(anyhow!("Couldn't determine path of data directory"))?;
        let sources = data.join("sources");
        let state = data.join("state");

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

pub fn load(config_dir: Option<PathBuf>) -> Result<()> {
    PATHS
        .set(Paths::load(config_dir)?)
        .ok()
        .expect("`paths::load` should only be called once");
    Ok(())
}

fn paths() -> &'static Paths {
    PATHS
        .get()
        .expect("`paths::load` should be called without failing before other functions in `paths` are called")
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

const TILDE: &str = "~";

/// Converts the home directory at the beginning of `path`, if it exists, into
/// a tilde.
///
/// The home directory is determined using `home`.
fn tildefy_with<P>(path: &Path, home: P) -> Cow<Path>
where
    P: AsRef<Path>,
{
    let home = home.as_ref();
    match path.strip_prefix(home) {
        Ok(path) => match path.parent() {
            None => Cow::Borrowed(Path::new(TILDE)),
            Some(_) => Cow::Owned(Path::new(TILDE).join(path)),
        },
        Err(_) => Cow::Borrowed(path),
    }
}

/// Converts the home directory at the beginning of `path`, if it exists, into
/// a tilde.
///
/// The home directory is determined using `paths::home`.
pub fn tildefy(path: &Path) -> Cow<Path> {
    tildefy_with(path, home())
}

/// Converts the tilde at the beginning of `path`, if it exists, into the home
/// directory.
///
/// The home directory is determined using `home`.
fn untildefy_with<P>(path: &Path, home: P) -> Cow<Path>
where
    P: AsRef<Path>,
{
    let home = home.as_ref();
    match path.strip_prefix(TILDE) {
        Ok(path) => Cow::Owned(home.join(path)),
        Err(_) => Cow::Borrowed(path),
    }
}

/// Converts the tilde at the beginning of `path`, if it exists, into the home
/// directory.
///
/// The home directory is determined using `paths::home`.
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
