use std::path::Path;

use anyhow::{Context, Result, bail};
use bincode::{Decode, Encode};
use serde::Deserialize;

use crate::{env::Env, utils::sha256::Sha256Hash};

use super::Source;

#[derive(Clone, Decode, Deserialize, Encode, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(deny_unknown_fields)]
pub struct HashableSource {
    #[serde(flatten)]
    source: Source,
    hash: Option<Sha256Hash>,
}

impl HashableSource {
    pub fn fetch(&self, env: &Env, path: &Path) -> Result<()> {
        self.source.fetch(env, path)?;
        self.check(path)?;
        Ok(())
    }

    pub fn check(&self, path: &Path) -> Result<()> {
        if let Some(hash) = &self.hash {
            if Sha256Hash::from_path(path).context("Couldn't calculate hash")? != *hash {
                bail!("Contents don't match hash");
            }
        }
        Ok(())
    }
}
