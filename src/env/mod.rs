use std::{
    borrow::Cow,
    ops::Deref,
    path::{Path, PathBuf},
};

use anyhow::Result;
use paths::Paths;
use users::{User, Users};

pub mod paths;
pub mod users;

pub struct Env {
    paths: Paths,
    users: Users,
}

impl Env {
    pub fn load(config_dir: Option<PathBuf>) -> Result<Self> {
        Ok(Env {
            paths: Paths::load(config_dir)?,
            users: Users::default(),
        })
    }

    /// Returns user with name `name` if that user isn't the current user.
    pub fn other_user_with_name(&mut self, name: &str) -> Result<Option<&User>> {
        let current_uid = self.users.uid();
        Ok(Some(self.users.user_with_name(name)?).filter(|user| user.uid != current_uid))
    }

    /// Returns user with UID `uid` if that user isn't the current user.
    pub fn other_user_with_uid(&mut self, uid: u32) -> Result<Option<&User>> {
        let current_uid = self.users.uid();
        Ok(Some(self.users.user_with_uid(uid)?).filter(|user| user.uid != current_uid))
    }

    /// Returns GID of group with name `name` if that group isn't the current
    /// group.
    pub fn other_group_gid(&mut self, name: &str) -> Result<Option<u32>> {
        let current_gid = self.users.gid();
        Ok(Some(self.users.group_gid(name)?).filter(|gid| *gid != current_gid))
    }

    const TILDE: &str = "~";

    pub fn tildefy<'a>(&self, path: &'a Path) -> Cow<'a, Path> {
        match path.strip_prefix(self.home_dir()) {
            Ok(path) => match path.parent() {
                None => Cow::Borrowed(Path::new(Self::TILDE)),
                Some(_) => Cow::Owned(Path::new(Self::TILDE).join(path)),
            },
            Err(_) => Cow::Borrowed(path),
        }
    }

    pub fn untildefy<'a>(&mut self, path: &'a Path) -> Result<Cow<'a, Path>> {
        let mut components = path.components();
        if let Some(prefix) = components.next() {
            let prefix = prefix.as_os_str().to_string_lossy();
            if let Some(user) = prefix.strip_prefix(Self::TILDE) {
                let path = match user {
                    "" => self.home_dir(),
                    name => self.users.user_with_name(name)?.home.as_path(),
                }
                .join(components);
                return Ok(Cow::Owned(path));
            }
        }
        Ok(Cow::Borrowed(path))
    }
}

impl Deref for Env {
    type Target = Paths;

    fn deref(&self) -> &Self::Target {
        &self.paths
    }
}
