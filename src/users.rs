use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use nix::unistd::{self, Uid};

struct User {
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

pub struct Users {
    default_uid: Uid,
    current_uid: Uid,
    users: HashMap<String, Option<User>>,
}

impl Users {
    pub fn new() -> Self {
        let uid = unistd::geteuid();
        Users {
            default_uid: uid,
            current_uid: uid,
            users: HashMap::new(),
        }
    }

    fn user(&mut self, name: &str) -> Result<&User> {
        if !self.users.contains_key(name) {
            let user = unistd::User::from_name(name)?.map(|user| user.into());
            self.users.insert(name.to_string(), user);
        }
        self.users
            .get(name)
            .expect("At this point, this entry should always exist")
            .as_ref()
            .ok_or(anyhow!("User doesn't exist"))
    }

    fn uid(&mut self, name: &str) -> Result<Uid> {
        self.user(name).map(|user| user.uid)
    }

    fn home(&mut self, name: &str) -> Result<&Path> {
        self.user(name).map(|user| user.home.as_path())
    }

    pub fn become_user(&mut self, name: &str) -> Result<bool> {
        let uid = self.uid(name)?;
        Ok(if uid != self.current_uid {
            unistd::seteuid(uid)?;
            self.current_uid = uid;
            true
        } else {
            false
        })
    }

    pub fn become_default_user(&mut self) -> Result<bool> {
        Ok(if self.current_uid != self.default_uid {
            unistd::seteuid(self.default_uid)?;
            self.current_uid = self.default_uid;
            true
        } else {
            false
        })
    }
}
