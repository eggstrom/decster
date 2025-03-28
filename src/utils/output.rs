use std::{
    fmt::{self, Display, Formatter},
    path::Path,
};

use crossterm::style::Stylize;

use crate::{global::paths, state::path::PathKind};

#[macro_export]
macro_rules! out {
    ($indent:expr, fetched, $($args:tt)*) => { out!($indent, "Fetched: ".green(), $($args)*) };
    ($indent:expr, created, $($args:tt)*) => { out!($indent, "Created: ".green(), $($args)*) };
    ($indent:expr, removed, $($args:tt)*) => { out!($indent, "Removed: ".green(), $($args)*) };
    ($indent:expr, skipped, $($args:tt)*) => { out!($indent, "Skipped: ".yellow(), $($args)*) };
    ($indent:expr, failed, $($args:tt)*) => { out!($indent, "Failed: ".red(), $($args)*) };
    ($indent:expr, $msg:expr, $($args:tt)*) => {{
        if !config::quiet() {
            (0..$indent).for_each(|_| print!("  "));
            print!("{}", $msg);
            println!($($args)*);
        }
    }};
}

pub struct DisplayFile<'a>(&'a Path);

impl Display for DisplayFile<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(file) = self.0.file_name() {
            if let Some(path) = self.0.parent() {
                write!(f, "{}/", paths::tildefy(path).display())?;
            }
            file.to_string_lossy().magenta().fmt(f)?;
        }
        Ok(())
    }
}

pub struct DisplayDir<'a>(&'a Path);

impl Display for DisplayDir<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}{}", self.0.display_file(), "/".magenta())
    }
}

pub struct DisplayKind<'a> {
    path: &'a Path,
    kind: PathKind,
}

impl Display for DisplayKind<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.kind {
            PathKind::Directory => self.path.display_dir().fmt(f),
            _ => self.path.display_file().fmt(f),
        }
    }
}

pub trait PathExt {
    fn display_file(&self) -> DisplayFile;
    fn display_dir(&self) -> DisplayDir;
    fn display_kind(&self, kind: PathKind) -> DisplayKind;
}

impl PathExt for Path {
    fn display_file(&self) -> DisplayFile {
        DisplayFile(self)
    }

    fn display_dir(&self) -> DisplayDir {
        DisplayDir(self)
    }

    fn display_kind(&self, kind: PathKind) -> DisplayKind {
        DisplayKind { path: self, kind }
    }
}
