use std::{
    borrow::Cow,
    ops::Deref,
    path::{Path, PathBuf},
};

use anyhow::Result;
use dirs::Dirs;
use users::{User, Users};

pub mod dirs;
pub mod users;

pub struct Env {
    dirs: Dirs,
    users: Users,
}

impl Env {
    pub fn load(config_dir: Option<PathBuf>) -> Result<Self> {
        Ok(Env {
            dirs: Dirs::load(config_dir)?,
            users: Users::default(),
        })
    }

    /// Returns user with name `name` if that user isn't the current user.
    pub fn other_user(&mut self, name: &str) -> Result<Option<&User>> {
        let current_uid = self.users.uid();
        Ok(Some(self.users.user(name)?).filter(|user| user.uid != current_uid))
    }

    /// Returns GID of group with name `name` if that group isn't the current
    /// group.
    pub fn other_group(&mut self, name: &str) -> Result<Option<u32>> {
        let current_gid = self.users.gid();
        Ok(Some(self.users.gid_of(name)?).filter(|gid| *gid != current_gid))
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
        Self::tildefy_with(path, &self.dirs.home())
    }

    pub fn untildefy<'a>(&self, path: &'a Path) -> Cow<'a, Path> {
        Self::untildefy_with(path, &self.home())
    }
}

impl Deref for Env {
    type Target = Dirs;

    fn deref(&self) -> &Self::Target {
        &self.dirs
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
        let home = ::dirs::home_dir().unwrap();
        for (tilde, untilde) in paths(&home) {
            assert_eq!(Env::tildefy_with(&untilde, &home), tilde);
        }
    }

    #[test]
    fn untildefy() {
        let home = ::dirs::home_dir().unwrap();
        for (tilde, no_tilde) in paths(&home) {
            assert_eq!(Env::untildefy_with(tilde, &home), no_tilde);
        }
    }
}
