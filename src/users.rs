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
    uid: Uid,
    users: HashMap<String, Option<User>>,
}

impl Users {
    pub fn new() -> Self {
        Users {
            uid: unistd::getuid(),
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

    pub fn uid(&mut self, name: &str) -> Result<Uid> {
        self.user(name).map(|user| user.uid)
    }

    fn home(&mut self, name: &str) -> Result<&Path> {
        self.user(name).map(|user| user.home.as_path())
    }

    pub fn is_current(&mut self, name: &str) -> bool {
        self.uid(name).is_ok_and(|uid| uid == self.uid)
    }
}
