use std::{
    borrow::Borrow,
    fmt::{self, Display, Formatter},
    fs,
    path::Path,
};

use crate::{source::SourcePath, utils};
use anyhow::{Context, Result, anyhow};
use crossterm::style::{Color, Stylize};
use serde::Deserialize;

pub struct IncompleteLink<'a> {
    pub path: &'a Path,
    pub source: &'a SourcePath,
}

impl<'a> IncompleteLink<'a> {
    pub fn new(path: &'a Path, source: &'a SourcePath) -> Self {
        IncompleteLink { path, source }
    }

    pub fn with_method(self, method: LinkMethod) -> Link<'a> {
        Link { link: self, method }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub enum LinkMethod {
    #[serde(rename = "copy")]
    Copy,
    #[default]
    #[serde(rename = "soft-link")]
    SoftLink,
    #[serde(rename = "hard-link")]
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
    link: IncompleteLink<'a>,
    method: LinkMethod,
}

impl Link<'_> {
    pub fn path(&self) -> &Path {
        self.link.path
    }

    pub fn source(&self) -> &SourcePath {
        self.link.source
    }

    pub fn source_name(&self) -> &str {
        self.source().name.borrow()
    }

    pub fn method(&self) -> LinkMethod {
        self.method
    }

    pub fn color(&self) -> Color {
        self.method().color()
    }

    pub fn exists(&self) -> Result<bool> {
        match self.method {
            LinkMethod::SoftLink => self.soft_link_exists(),
            _ => self.file_exists(),
        }
    }

    pub fn file_exists(&self) -> Result<bool> {
        utils::all_files_match(self.path(), self.source().path()?)
    }

    pub fn soft_link_exists(&self) -> Result<bool> {
        let path = self.path();
        Ok(path.is_symlink() && path.read_link()? == self.source().path()?)
    }

    pub fn enable(&self) -> Result<()> {
        if let Some(dirs) = self.path().parent() {
            fs::create_dir_all(dirs)?;
        }
        match self.method() {
            LinkMethod::Copy => self.enable_copy(),
            LinkMethod::HardLink => self.enable_hard_link(),
            LinkMethod::SoftLink => self.enable_soft_link(),
        }
        .with_context(|| anyhow!("couldn't create link: {self}"))
    }

    pub fn enable_copy(&self) -> Result<()> {
        fs::rename(self.source().path()?, self.path())?;
        Ok(())
    }

    pub fn enable_hard_link(&self) -> Result<()> {
        todo!()
    }

    pub fn enable_soft_link(&self) -> Result<()> {
        todo!()
    }
}

impl Display for Link<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.path().display(),
            "->".with(self.color()),
            self.source(),
            self.method()
        )
    }
}
