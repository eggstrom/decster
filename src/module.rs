use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use anyhow::Result;
use serde::Deserialize;

use crate::{
    config::Config,
    link::{Link, method::LinkMethod},
    source::path::SourcePath,
    state::State,
};

#[derive(Debug, Deserialize)]
pub struct Module {
    #[serde(default)]
    import: HashSet<String>,
    link_method: Option<LinkMethod>,
    #[serde(default)]
    links: HashMap<PathBuf, SourcePath>,
}

impl Module {
    pub fn links<'a>(
        &'a self,
        name: &'a str,
        default_method: LinkMethod,
    ) -> impl Iterator<Item = Link<'a>> {
        self.links.iter().map(move |(path, source)| {
            Link::new(
                name,
                path,
                source,
                self.link_method.unwrap_or(default_method),
            )
        })
    }

    pub fn add_sources(&self, config: &Config, state: &mut State) -> Result<()> {
        for source in self.links.values() {
            let name = &source.name;
            let source = config.source(name)?;
            state.add_source(name, source)?;
        }
        Ok(())
    }

    pub fn enable(&self, state: &mut State, name: &str, default_method: LinkMethod) -> Result<()> {
        for link in self.links(name, default_method) {
            link.enable(state)?;
        }
        Ok(())
    }
}

pub enum ModuleFilter {
    All,
    Enabled,
    Disabled,
}
