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

pub struct Users(HashMap<String, Option<User>>);

impl Users {
    pub fn new() -> Self {
        Users(HashMap::new())
    }

    fn user(&mut self, name: &str) -> Result<&User> {
        if !self.0.contains_key(name) {
            let user = unistd::User::from_name(name)?.map(|user| user.into());
            self.0.insert(name.to_string(), user);
        }
        self.0
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
}
