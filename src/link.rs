use std::{
    borrow::Borrow,
    fmt::{self, Display, Formatter},
    path::Path,
};

use crate::source::SourcePath;
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
}

impl Display for Link<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.source(),
            "->".with(self.color()),
            self.path().display(),
            self.method()
        )
    }
}
