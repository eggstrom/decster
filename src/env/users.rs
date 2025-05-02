use std::{collections::HashMap, path::PathBuf};

use anyhow::{Result, anyhow};
use crossterm::style::Stylize;
use nix::unistd::{self, Group};

pub struct User {
    pub uid: u32,
    pub gid: u32,
    pub home: PathBuf,
}

impl User {
    fn new(name: &str) -> Result<Option<Self>> {
        Ok(unistd::User::from_name(name)?.map(|user| User {
            uid: user.uid.as_raw(),
            gid: user.gid.as_raw(),
            home: user.dir,
        }))
    }
}

#[derive(Default)]
pub struct Users {
    uid: Option<u32>,
    gid: Option<u32>,
    users: HashMap<String, Option<User>>,
    groups: HashMap<String, Option<u32>>,
}

impl Users {
    pub fn uid(&mut self) -> u32 {
        *self.uid.get_or_insert_with(|| unistd::getuid().as_raw())
    }

    pub fn gid(&mut self) -> u32 {
        *self.gid.get_or_insert_with(|| unistd::getgid().as_raw())
    }

    pub fn user<'a>(&'a mut self, name: &str) -> Result<&'a User> {
        if !self.users.contains_key(name) {
            let user = User::new(name)?;
            self.users.insert(name.to_string(), user);
        }
        self.users
            .get(name)
            .unwrap()
            .as_ref()
            .ok_or_else(|| anyhow!("User `{}` doesn't exist", name.magenta()))
    }

    pub fn group_gid(&mut self, name: &str) -> Result<u32> {
        if !self.groups.contains_key(name) {
            let group = Group::from_name(name)?.map(|group| group.gid.as_raw());
            self.groups.insert(name.to_string(), group);
        }
        self.groups
            .get(name)
            .unwrap()
            .ok_or(anyhow!("Group `{}` doesn't exist", name.magenta()))
    }
}
