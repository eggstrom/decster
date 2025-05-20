use std::{
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
};

use bincode::{Decode, Encode};
use crossterm::style::Stylize;
use sha2::{Digest, Sha256};

use crate::{
    env::Env,
    globs::Globs,
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

    pub fn matches_globs(&self, globs: &Globs) -> bool {
        match self {
            SourceIdent::Named(name) => globs.is_match(name),
            SourceIdent::Unnamed { module, path } => globs.is_match(module) || globs.is_match(path),
        }
    }

    pub fn path(&self, env: &Env) -> PathBuf {
        match self {
            SourceIdent::Named(name) => env.named_source_dir().join(name),
            SourceIdent::Unnamed { module, path } => env.unnamed_source_dir().join({
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
