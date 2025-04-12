use std::{
    fmt::{self, Display, Formatter},
    path::Path,
};

use crossterm::style::Stylize;

use crate::paths;

pub struct PrettyPath<'a>(&'a Path);

impl Display for PrettyPath<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(file) = self.0.file_name() {
            if let Some(path) = self.0.parent() {
                paths::tildefy(path).display().fmt(f)?;
                if path.parent().is_some() {
                    "/".fmt(f)?;
                }
            }
            file.to_string_lossy().magenta().fmt(f)?;
        }
        Ok(())
    }
}

pub trait PrettyPathExt {
    fn pretty(&self) -> PrettyPath;
}

impl PrettyPathExt for Path {
    fn pretty(&self) -> PrettyPath {
        PrettyPath(self)
    }
}

pub struct PrettyStrSlice<'a, S>(&'a [S])
where
    S: AsRef<str>;

impl<S> Display for PrettyStrSlice<'_, S>
where
    S: AsRef<str>,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (i, item) in self.0.iter().enumerate() {
            write!(f, "'{}'", item.as_ref().yellow())?;
            match self.0.len().checked_sub(2) {
                Some(x) if i < x => ", ".fmt(f)?,
                Some(x) if i == x => " and ".fmt(f)?,
                _ => (),
            };
        }
        Ok(())
    }
}

pub trait PrettyStrSliceExt<S>
where
    S: AsRef<str>,
{
    fn pretty(&self) -> PrettyStrSlice<S>;
}

impl<S, T> PrettyStrSliceExt<S> for T
where
    T: AsRef<[S]>,
    S: AsRef<str>,
{
    fn pretty(&self) -> PrettyStrSlice<S> {
        PrettyStrSlice(self.as_ref())
    }
}
