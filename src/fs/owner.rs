use std::{
    fmt::{self, Display, Formatter},
    fs::Metadata,
    os::unix::{self, fs::MetadataExt},
    path::Path,
    str::FromStr,
};

use anyhow::{Context, Result};
use derive_more::From;
use serde::{
    Deserialize, Deserializer,
    de::{self, Visitor},
};
use thiserror::Error;

use crate::{env::Env, utils::pretty::Pretty};

#[derive(Debug, Default, PartialEq)]
pub struct Owner {
    user: Option<OwnerIdent>,
    group: Option<OwnerGroup>,
}

#[derive(Debug, From, PartialEq)]
enum OwnerGroup {
    #[from(forward)]
    Ident(OwnerIdent),
    LoginGroup,
}

#[derive(Debug, PartialEq)]
enum OwnerIdent {
    Id(u32),
    Name(String),
}

impl From<&str> for OwnerIdent {
    fn from(s: &str) -> Self {
        match s.parse() {
            Ok(id) => OwnerIdent::Id(id),
            Err(_) => OwnerIdent::Name(s.to_string()),
        }
    }
}

impl Owner {
    fn new(user: Option<OwnerIdent>, group: Option<OwnerGroup>) -> Self {
        Owner { user, group }
    }

    fn user(user: impl Into<OwnerIdent>) -> Self {
        Owner::new(Some(user.into()), None)
    }

    fn user_with_login_group(user: impl Into<OwnerIdent>) -> Self {
        Owner::new(Some(user.into()), Some(OwnerGroup::LoginGroup))
    }

    fn user_with_group(user: impl Into<OwnerIdent>, group: impl Into<OwnerGroup>) -> Self {
        Owner::new(Some(user.into()), Some(group.into()))
    }

    fn group(group: impl Into<OwnerGroup>) -> Self {
        Owner::new(None, Some(group.into()))
    }

    pub fn ids(&self, env: &mut Env) -> Result<OwnerIds> {
        let user = match &self.user {
            Some(OwnerIdent::Id(uid)) => env.other_user_with_uid(*uid)?,
            Some(OwnerIdent::Name(name)) => env.other_user_with_name(name)?,
            None => None,
        };
        let uid = user.as_ref().map(|user| user.uid);
        let gid = match &self.group {
            Some(OwnerGroup::LoginGroup) => user.as_ref().map(|user| user.gid),
            Some(OwnerGroup::Ident(OwnerIdent::Id(gid))) => Some(*gid),
            Some(OwnerGroup::Ident(OwnerIdent::Name(name))) => env.other_group_gid(name)?,
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
            Some(("", "")) => Owner::default(),
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
    pub fn from_metadata(metadata: &Metadata) -> Self {
        let (uid, gid) = (Some(metadata.uid()), Some(metadata.gid()));
        OwnerIds { uid, gid }
    }

    pub fn set(&self, env: &Env, path: &Path) -> Result<()> {
        if self.uid.is_none() && self.gid.is_none() {
            return Ok(());
        }
        unix::fs::lchown(path, self.uid, self.gid)
            .with_context(|| format!("Couldn't set owner of {}", env.tildefy(path).pretty()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rustfmt::skip]
    #[test]
    pub fn parse() {
        assert_eq!(Owner::from_str(":"), Ok(Owner::default()));
        assert_eq!(Owner::from_str("user:"), Ok(Owner::user_with_login_group("user")));
        assert_eq!(Owner::from_str("user:group"), Ok(Owner::user_with_group("user", "group")));
        assert_eq!(Owner::from_str(":group"), Ok(Owner::group("group")));
        assert_eq!(Owner::from_str(""), Err(ParseOwnerError));
    }
}
