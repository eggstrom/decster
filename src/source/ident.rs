use std::{
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
};

use bincode::{Decode, Encode};
use crossterm::style::Stylize;
use sha2::{Digest, Sha256};

use crate::{
    global::env,
    utils::{pretty::Pretty, sha256::Sha256Hash},
};

use super::name::SourceName;

#[derive(Clone, Decode, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub enum SourceIdent {
    Named(SourceName),
    Unnamed { module: String, path: PathBuf },
}

impl SourceIdent {
    pub fn named(name: SourceName) -> Self {
        SourceIdent::Named(name)
    }

    pub fn unnamed(module: &str, path: &Path) -> Self {
        SourceIdent::Unnamed {
            module: module.to_string(),
            path: path.to_path_buf(),
        }
    }

    pub fn is_named_and<F>(&self, f: F) -> bool
    where
        F: FnOnce(&SourceName) -> bool,
    {
        match self {
            SourceIdent::Named(name) => f(name),
            _ => false,
        }
    }

    pub fn path(&self) -> PathBuf {
        match self {
            SourceIdent::Named(name) => env::named_sources().join(name),
            SourceIdent::Unnamed { module, path } => env::unnamed_sources().join({
                let mut hasher = Sha256::new();
                hasher.update(module);
                hasher.update(path.to_string_lossy().as_ref());
                Sha256Hash::from(hasher.finalize()).to_string()
            }),
        }
    }
}

impl Display for SourceIdent {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            SourceIdent::Named(name) => name.fmt(f),
            SourceIdent::Unnamed { module, path } => {
                let module = module.as_str().magenta();
                write!(f, "{} in {module}", path.pretty())
            }
        }
    }
}
