use std::fmt::{self, Display, Formatter};

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
