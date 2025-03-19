use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};
use serde::Deserialize;

use crate::{
    link::{IncompleteLink, LinkMethod},
    source::SourcePath,
    utils,
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

    pub fn enable<P>(&self, default_method: LinkMethod, data_dir: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let data_dir = data_dir.as_ref();
        let mut created_files = Vec::new();

        for link in self
            .links()
            .map(|(link, method)| link.with_method(method.unwrap_or(default_method)))
        {
            if let Some(dirs) = link.path().parent() {
                fs::create_dir_all(dirs)?;
            }
            match link.enable(data_dir) {
                Ok(()) => created_files.push(link.path().to_path_buf()),
                Err(error) => {
                    for path in created_files {
                        utils::remove_all(path)?;
                    }
                    bail!(error);
                }
            }
        }
        Ok(())
    }

    pub fn disable(&self) {}
}
