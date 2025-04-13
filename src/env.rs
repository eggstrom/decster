use std::{
    borrow::Cow,
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use nix::unistd::{self, Uid};

pub struct User {
    uid: Uid,
    home: PathBuf,
}

impl From<unistd::User> for User {
    fn from(value: unistd::User) -> Self {
        User {
            uid: value.uid,
            home: value.dir,
        }
    }
}

pub struct Env {
    uid: Uid,
    users: HashMap<String, Option<User>>,

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
    const TILDE: &str = "~";

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
            uid: unistd::geteuid(),
            users: HashMap::new(),

            home,
            config: config.join("config.toml"),
            modules: config.join("modules"),
            config_sources: config.join("sources"),
            named_sources,
            unnamed_sources,
            state: data.join("state"),
        })
    }

    pub fn is_current_uid<U>(&mut self, uid: U) -> bool
    where
        U: Into<Uid>,
    {
        uid.into() == self.uid
    }

    fn user(&mut self, name: &str) -> Result<&User> {
        if !self.users.contains_key(name) {
            let user = unistd::User::from_name(name)?.map(|user| user.into());
            self.users.insert(name.to_string(), user);
        }
        self.users
            .get(name)
            .unwrap()
            .as_ref()
            .ok_or(anyhow!("User doesn't exist"))
    }

    pub fn uid(&mut self, name: &str) -> Result<Uid> {
        self.user(name).map(|user| user.uid)
    }

    fn home_of(&mut self, name: &str) -> Result<&Path> {
        self.user(name).map(|user| user.home.as_path())
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

    pub fn config_sources(&self) -> &Path {
        &self.config_sources
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

    fn tildefy_with<'a>(&self, path: &'a Path, home: &Path) -> Cow<'a, Path> {
        match path.strip_prefix(home) {
            Ok(path) => match path.parent() {
                None => Cow::Borrowed(Path::new(Self::TILDE)),
                Some(_) => Cow::Owned(Path::new(Self::TILDE).join(path)),
            },
            Err(_) => Cow::Borrowed(path),
        }
    }

    pub fn tildefy<'a>(&self, path: &'a Path) -> Cow<'a, Path> {
        self.tildefy_with(path, self.home())
    }

    fn untildefy_with<'a>(&self, path: &'a Path, home: &Path) -> Cow<'a, Path> {
        match path.strip_prefix(Self::TILDE) {
            Ok(path) => Cow::Owned(home.join(path)),
            Err(_) => Cow::Borrowed(path),
        }
    }

    pub fn untildefy<'a>(&self, path: &'a Path) -> Cow<'a, Path> {
        self.untildefy_with(path, self.home())
    }
}
