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
        self.source.name.as_ref()
    }

    pub fn is_enabled(&self) -> Result<bool> {
        match self.method {
            LinkMethod::SoftLink => self.is_soft_link_enabled(),
            _ => self.is_file_enabled(),
        }
    }

    fn is_file_enabled(&self) -> Result<bool> {
        utils::all_match(self.path, self.source.path()?)
    }

    fn is_soft_link_enabled(&self) -> Result<bool> {
        let path = self.path;
        Ok(path.is_symlink() && path.read_link()? == self.source.path()?)
    }

    pub fn enable(&self) -> Result<()> {
        if let Some(dirs) = self.path.parent() {
            fs::create_dir_all(dirs)?;
        }
        match self.method {
            LinkMethod::Copy => self.enable_copy(),
            LinkMethod::HardLink => self.enable_hard_link(),
            LinkMethod::SoftLink => self.enable_soft_link(),
        }
        .with_context(|| anyhow!("couldn't create link: {self}"))
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
        match self.method {
            LinkMethod::SoftLink => self.disable_soft_link(),
            _ => self.disable_file(),
        }
    }

    fn disable_file(&self) -> Result<()> {
        Ok(())
    }

    fn disable_soft_link(&self) -> Result<()> {
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
