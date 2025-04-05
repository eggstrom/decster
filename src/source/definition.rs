use std::{io, path::PathBuf};

use bincode::{Decode, Encode};
use serde::Deserialize;

use crate::{
    config, out, paths,
    state::State,
    utils::sha256::{PathHash, Sha256Hash},
};

use super::{Source, name::SourceName};

#[derive(Clone, Decode, Deserialize, Encode, PartialEq)]
#[serde(untagged)]
pub enum SourceDefinition {
    #[serde(skip)]
    Static,
    Dynamic {
        #[serde(flatten)]
        source: Source,
        hash: Option<Sha256Hash>,
    },
}

impl SourceDefinition {
    pub fn path(&self, name: &SourceName) -> PathBuf {
        match self {
            SourceDefinition::Static => paths::config().join("sources").join(name),
            SourceDefinition::Dynamic { .. } => paths::sources().join(name),
        }
    }

    pub fn hash(&self, name: &SourceName) -> io::Result<Sha256Hash> {
        self.path(name).hash_all()
    }

    pub fn fetch_and_verify(&self, state: &mut State, name: &SourceName) {
        let (source, hash) = match self {
            SourceDefinition::Static => return,
            SourceDefinition::Dynamic { source, .. }
                if !config::fetch() && state.has_source(name, self) =>
            {
                out!(2, Y; "{name}"; "{source}");
                return;
            }
            SourceDefinition::Dynamic { source, hash } => (source, hash),
        };
        if let Err(err) = source.fetch(name) {
            out!(2, R; "{name}"; "{err}");
            return;
        };
        if let Some(hash) = hash {
            if !name.path().hash_all().is_ok_and(|h| h == *hash) {
                out!(2, R; "{name}"; "Contents don't match hash");
                return;
            }
        }
        out!(2, G; "{name}"; "{source}");
        state.add_source(name, self);
    }
}
