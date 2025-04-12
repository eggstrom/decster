use std::{
    fmt::{self, Display, Formatter},
    path::Path,
};

use anyhow::{Context, Result, bail};
use bincode::{Decode, Encode};
use serde::Deserialize;

use crate::utils::sha256::{PathHash, Sha256Hash};

use super::Source;

#[derive(Clone, Decode, Deserialize, Encode, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct HashableSource {
    #[serde(flatten)]
    source: Source,
    hash: Option<Sha256Hash>,
}

impl HashableSource {
    pub fn fetch(&self, path: &Path) -> Result<()> {
        self.source.fetch(path)?;
        if let Some(hash) = &self.hash {
            if path.hash_all().context("Couldn't calculate hash")? == *hash {
                bail!("Contents don't match hash");
            }
        }
        Ok(())
    }
}

impl Display for HashableSource {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.source.fmt(f)
    }
}
