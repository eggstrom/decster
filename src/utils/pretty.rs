use std::{
    fmt::{self, Display, Formatter},
    path::Path,
};

use crossterm::style::Stylize;

use crate::paths;

pub trait Pretty {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result;

    fn pretty<'a>(&'a self) -> PrettyRef<'a, Self> {
        PrettyRef(self)
    }
}

pub struct PrettyRef<'a, T>(&'a T)
where
    T: Pretty + ?Sized;

impl<T> Display for PrettyRef<'_, T>
where
    T: Pretty + ?Sized,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Pretty::fmt(self.0, f)
    }
}

impl Pretty for Path {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(file) = self.file_name() {
            if let Some(path) = self.parent() {
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

impl<T> Pretty for &[T]
where
    T: AsRef<str>,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (i, item) in self.iter().enumerate() {
            write!(f, "'{}'", item.as_ref().yellow())?;
            match self.len().checked_sub(2) {
                Some(x) if i < x => ", ".fmt(f)?,
                Some(x) if i == x => " and ".fmt(f)?,
                _ => (),
            };
        }
        Ok(())
    }
}
