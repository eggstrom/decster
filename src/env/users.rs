use std::{collections::HashMap, path::PathBuf, rc::Rc};

use anyhow::{Result, anyhow};
use crossterm::style::Stylize;
use nix::unistd::{self, Group, Uid};

pub struct User {
    pub uid: u32,
    pub gid: u32,
    pub home: PathBuf,
}

impl User {
    fn from_unistd(user: unistd::User) -> (String, Rc<Self>) {
        let name = user.name;
        let user = User {
            uid: user.uid.as_raw(),
            gid: user.gid.as_raw(),
            home: user.dir,
        };
        (name, Rc::new(user))
    }

    pub fn from_name(name: &str) -> Result<Option<(String, Rc<Self>)>> {
        Ok(unistd::User::from_name(name)?.map(User::from_unistd))
    }

    pub fn from_uid(uid: u32) -> Result<Option<(String, Rc<Self>)>> {
        Ok(unistd::User::from_uid(Uid::from_raw(uid))?.map(User::from_unistd))
    }
}

#[derive(Default)]
pub struct Users {
    uid: Option<u32>,
    gid: Option<u32>,
    users_by_name: HashMap<String, Option<Rc<User>>>,
    users_by_uid: HashMap<u32, Option<Rc<User>>>,
    group_gids: HashMap<String, Option<u32>>,
}

impl Users {
    pub fn uid(&mut self) -> u32 {
        *self.uid.get_or_insert_with(|| unistd::getuid().as_raw())
    }

    pub fn gid(&mut self) -> u32 {
        *self.gid.get_or_insert_with(|| unistd::getgid().as_raw())
    }

    pub fn user_by_name<'a>(&'a mut self, name: &str) -> Result<&'a User> {
        if !self.users_by_name.contains_key(name) {
            if let Some((name, user)) = User::from_name(name)? {
                self.users_by_name.insert(name, Some(Rc::clone(&user)));
                self.users_by_uid.insert(user.uid, Some(user));
            } else {
                self.users_by_name.insert(name.to_string(), None);
            }
        }
        self.users_by_name
            .get(name)
            .unwrap()
            .as_deref()
            .ok_or_else(|| anyhow!("User `{}` doesn't exist", name.magenta()))
    }

    pub fn user_by_uid(&mut self, uid: u32) -> Result<&User> {
        if !self.users_by_uid.contains_key(&uid) {
            if let Some((name, user)) = User::from_uid(uid)? {
                self.users_by_name.insert(name, Some(Rc::clone(&user)));
                self.users_by_uid.insert(user.uid, Some(user));
            } else {
                self.users_by_uid.insert(uid, None);
            }
        }
        self.users_by_uid
            .get(&uid)
            .unwrap()
            .as_deref()
            .ok_or_else(|| anyhow!("User with UID {uid} doesn't exist"))
    }

    pub fn group_gid(&mut self, name: &str) -> Result<u32> {
        if !self.group_gids.contains_key(name) {
            let (name, gid) = match Group::from_name(name)? {
                Some(Group { name, gid, .. }) => (name, Some(gid.as_raw())),
                None => (name.to_string(), None),
            };
            self.group_gids.insert(name, gid);
        }
        self.group_gids
            .get(name)
            .unwrap()
            .ok_or(anyhow!("Group `{}` doesn't exist", name.magenta()))
    }
}
