use std::fmt::{self, Display, Formatter};

use bincode::{Decode, Encode};
use serde::Deserialize;

use crate::{
    out,
    state::State,
    utils::sha256::{PathHash, Sha256Hash},
};

use super::{Source, ident::SourceIdent};

#[derive(Clone, Decode, Deserialize, Encode, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct SourceInfo {
    #[serde(flatten)]
    source: Source,
    hash: Option<Sha256Hash>,
}

impl SourceInfo {
    pub fn fetch_and_verify(&self, state: &mut State, ident: &SourceIdent) {
        if let Err(err) = self.source.fetch(ident) {
            out!(2, R; "{ident}"; "{err}");
            return;
        };
        if let Some(hash) = &self.hash {
            if !ident.path().hash_all().is_ok_and(|h| h == *hash) {
                out!(2, R; "{ident}"; "Contents don't match hash");
                return;
            }
        }
        out!(2, G; "{ident}"; "{}", self.source);
        state.add_source(ident, self);
    }
}

impl Display for SourceInfo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.source.fmt(f)
    }
}
