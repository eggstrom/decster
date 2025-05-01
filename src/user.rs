use std::path::PathBuf;

use anyhow::{Result, anyhow};
use nix::unistd::{self, Group};

use crate::global::env::{self};

#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct User {
    pub uid: u32,
    pub gid: u32,
    home: PathBuf,
}

impl User {
    pub fn new(name: &str) -> Result<Self> {
        unistd::User::from_name(name)?
            .map(|user| User {
                uid: user.uid.as_raw(),
                gid: user.gid.as_raw(),
                home: user.dir,
            })
            .ok_or(anyhow!("User doesn't exist"))
    }

    pub fn is_current(&self) -> bool {
        self.uid == env::uid()
    }
}

pub fn gid(name: &str) -> Result<u32> {
    Ok(Group::from_name(name)?
        .ok_or(anyhow!("Group doesn't exist"))?
        .gid
        .as_raw())
}
