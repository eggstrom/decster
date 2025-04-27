use std::{
    fmt::{self, Display, Formatter},
    path::Path,
};

use anyhow::Result;
use globset::{GlobBuilder, GlobSet};
use serde::{
    Deserialize, Deserializer,
    de::{self, SeqAccess, Visitor},
};

#[derive(Default)]
pub struct Globs {
    set: GlobSet,
    match_if_empty: bool,
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

struct GlobsVisitor;

impl<'de> Visitor<'de> for GlobsVisitor {
    type Value = Globs;

    fn expecting(&self, f: &mut Formatter) -> fmt::Result {
        "a glob or a list of globs".fmt(f)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Globs::strict([v]).map_err(de::Error::custom)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut globs = Vec::new();
        while let Some(glob) = seq.next_element::<String>()? {
            globs.push(glob);
        }
        Globs::strict(globs).map_err(de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for Globs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(GlobsVisitor)
    }
}
