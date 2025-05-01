use std::{
    fmt::{self, Display, Formatter},
    os::unix,
    path::Path,
    str::FromStr,
};

use anyhow::{Context, Result};
use serde::{
    Deserialize, Deserializer,
    de::{self, Visitor},
};
use thiserror::Error;

use crate::{
    global::env,
    user::{self, User},
    utils::pretty::Pretty,
};

#[derive(Debug, PartialEq)]
pub struct Owner {
    user: Option<String>,
    group: Option<Group>,
}

#[derive(Debug, PartialEq)]
enum Group {
    Name(String),
    LoginGroup,
}

impl Owner {
    fn new(user: Option<String>, group: Option<Group>) -> Self {
        Owner { user, group }
    }

    fn empty() -> Self {
        Owner::new(None, None)
    }

    fn user(user: &str) -> Self {
        Owner::new(Some(user.to_string()), None)
    }

    fn user_with_login_group(user: &str) -> Self {
        Owner::new(Some(user.to_string()), Some(Group::LoginGroup))
    }

    fn user_with_group(user: &str, group: &str) -> Self {
        Owner::new(Some(user.to_string()), Some(Group::Name(group.to_string())))
    }

    fn group(group: &str) -> Self {
        Owner::new(None, Some(Group::Name(group.to_string())))
    }

    pub fn ids(&self) -> Result<OwnerIds> {
        let user = self
            .user
            .as_ref()
            .map(|name| User::new(name))
            .transpose()?
            .filter(|user| !user.is_current());
        let uid = user.as_ref().map(|user| user.gid);
        let gid = match &self.group {
            Some(Group::Name(name)) => Some(user::gid(name)?).filter(|gid| *gid != env::gid()),
            Some(Group::LoginGroup) => user.as_ref().map(|user| user.gid),
            None => None,
        };
        Ok(OwnerIds { uid, gid })
    }
}

#[derive(Error, Debug, PartialEq)]
#[error("Owner string is empty")]
pub struct ParseOwnerError;

impl FromStr for Owner {
    type Err = ParseOwnerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.split_once(':') {
            None if s.is_empty() => Err(ParseOwnerError)?,
            None => Owner::user(s),
            Some(("", "")) => Owner::empty(),
            Some((user, "")) => Owner::user_with_login_group(user),
            Some(("", group)) => Owner::group(group),
            Some((user, group)) => Owner::user_with_group(user, group),
        })
    }
}

struct OwnerVisitor;

impl Visitor<'_> for OwnerVisitor {
    type Value = Owner;

    fn expecting(&self, f: &mut Formatter) -> fmt::Result {
        "a string representing an owner and group".fmt(f)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Owner::from_str(v).map_err(de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for Owner {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(OwnerVisitor)
    }
}

#[derive(Clone, Copy, Eq, Ord, PartialOrd, PartialEq)]
pub struct OwnerIds {
    uid: Option<u32>,
    gid: Option<u32>,
}

impl OwnerIds {
    pub fn change_owner(&self, path: &Path) -> Result<()> {
        if self.uid.is_none() && self.gid.is_none() {
            return Ok(());
        }
        let uid = self.uid.map_or("null".to_string(), |uid| uid.to_string());
        let gid = self.gid.map_or("null".to_string(), |gid| gid.to_string());
        println!("UID: {uid}, GID: {gid}");
        unix::fs::lchown(path, self.uid, self.gid)
            .with_context(|| format!("Couldn't change owner of {}", env::tildefy(path).pretty()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rustfmt::skip]
    #[test]
    pub fn parse() {
        assert_eq!(Owner::from_str(":"), Ok(Owner::empty()));
        assert_eq!(Owner::from_str("user:"), Ok(Owner::user_with_login_group("user")));
        assert_eq!(Owner::from_str("user:group"), Ok(Owner::user_with_group("user", "group")));
        assert_eq!(Owner::from_str(":group"), Ok(Owner::group("group")));
        assert_eq!(Owner::from_str(""), Err(ParseOwnerError));
    }
}
