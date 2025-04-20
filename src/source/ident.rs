use std::{
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
};

use bincode::{Decode, Encode};
use sha2::{Digest, Sha256};

use crate::{global::env, utils::sha256::Sha256Hash};

use super::name::SourceName;

#[derive(Clone, Decode, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub enum SourceIdent {
    Name(SourceName),
    Hash(Sha256Hash),
}

impl SourceIdent {
    pub fn named(name: SourceName) -> Self {
        SourceIdent::Name(name)
    }

    pub fn unnamed(module: &str, path: &Path) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(module);
        hasher.update(path.to_string_lossy().as_ref());
        SourceIdent::Hash(hasher.finalize().into())
    }

    pub fn path(&self) -> PathBuf {
        match self {
            SourceIdent::Name(name) => env::named_sources().join(name),
            SourceIdent::Hash(hash) => env::unnamed_sources().join(hash.to_string()),
        }
    }
}

impl Display for SourceIdent {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            SourceIdent::Name(name) => name.fmt(f),
            SourceIdent::Hash(_) => "Unnamed".fmt(f),
        }
    }
}
