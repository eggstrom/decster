use std::path::Path;

use anyhow::Result;
use globset::{GlobBuilder, GlobSet};
use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(try_from = "Vec<String>")]
pub struct Globs {
    set: GlobSet,
    match_if_empty: bool,
}

impl TryFrom<Vec<String>> for Globs {
    type Error = globset::Error;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        Self::strict(value)
    }
}

impl Globs {
    fn set<I>(globs: I) -> Result<GlobSet, globset::Error>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let mut set = GlobSet::builder();
        for glob in globs {
            let glob = GlobBuilder::new(glob.as_ref())
                .literal_separator(true)
                .build()?;
            set.add(glob);
        }
        set.build()
    }

    pub fn strict<I>(globs: I) -> Result<Globs, globset::Error>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        Ok(Globs {
            set: Self::set(globs)?,
            match_if_empty: false,
        })
    }

    pub fn permissive<I>(globs: I) -> Result<Self, globset::Error>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        Ok(Globs {
            set: Self::set(globs)?,
            match_if_empty: true,
        })
    }

    pub fn is_match<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        (self.match_if_empty && self.set.is_empty()) || self.set.is_match(path)
    }
}
