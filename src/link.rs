use std::{
    fmt::{self, Display, Formatter},
    fs,
    path::Path,
};

use crate::{source::SourcePath, utils};
use anyhow::{Context, Result, anyhow};
use crossterm::style::{Color, Stylize};
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkMethod {
    Copy,
    #[default]
    SoftLink,
    HardLink,
}

impl LinkMethod {
    pub fn color(&self) -> Color {
        match self {
            LinkMethod::Copy => Color::Green,
            LinkMethod::HardLink => Color::Cyan,
            LinkMethod::SoftLink => Color::Blue,
        }
    }
}

impl Display for LinkMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LinkMethod::Copy => "(copy)",
            LinkMethod::HardLink => "(hard link)",
            LinkMethod::SoftLink => "(soft link)",
        }
        .with(self.color())
        .fmt(f)
    }
}

pub struct Link<'a> {
    pub path: &'a Path,
    pub source: &'a SourcePath,
    pub method: LinkMethod,
}

impl<'a> Link<'a> {
    pub fn new(path: &'a Path, source: &'a SourcePath, method: LinkMethod) -> Self {
        Link {
            path,
            source,
            method,
        }
    }

    pub fn source_name(&self) -> &str {
        &self.source.name
    }

    pub fn enable(&self) -> Result<()> {
        if let Some(dirs) = self.path.parent() {
            fs::create_dir_all(dirs)
                .with_context(|| format!("Couldn't create path for link: {}", dirs.display()))?;
        }
        match self.method {
            LinkMethod::Copy => self.enable_copy(),
            LinkMethod::HardLink => self.enable_hard_link(),
            LinkMethod::SoftLink => self.enable_soft_link(),
        }
        .with_context(|| anyhow!("Couldn't create link: {self}"))
    }

    fn enable_copy(&self) -> Result<()> {
        utils::copy_all(self.source.path()?, self.path)?;
        Ok(())
    }

    fn enable_hard_link(&self) -> Result<()> {
        todo!()
    }

    fn enable_soft_link(&self) -> Result<()> {
        todo!()
    }

    pub fn disable(&self) -> Result<()> {
        utils::remove_all(self.path)?;
        utils::remove_dir_components(self.path);
        Ok(())
    }
}

impl Display for Link<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.path.display(),
            "->".with(self.method.color()),
            self.source,
            self.method
        )
    }
}
