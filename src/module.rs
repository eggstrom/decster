use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use serde::Deserialize;

use crate::{
    link::{IncompleteLink, LinkMethod},
    source::SourcePath,
};

#[derive(Debug, Deserialize)]
pub struct Module {
    #[serde(default)]
    pub import: HashSet<String>,
    pub link_method: Option<LinkMethod>,
    #[serde(default)]
    pub links: HashMap<PathBuf, SourcePath>,
}

impl Module {
    pub fn links(&self) -> impl Iterator<Item = (IncompleteLink, Option<LinkMethod>)> {
        self.links
            .iter()
            .map(|(path, source)| (IncompleteLink::new(path, source), self.link_method))
    }
}
