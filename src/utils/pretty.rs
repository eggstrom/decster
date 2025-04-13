use std::{
    fmt::{self, Display, Formatter},
    path::Path,
};

use crossterm::style::Stylize;

pub trait Pretty {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result;

    fn pretty(&self) -> PrettyRef<'_, Self> {
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
                path.display().fmt(f)?;
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
        for (i, string) in self.iter().enumerate() {
            write!(f, "'{}'", string.as_ref().yellow())?;
            match self.len().checked_sub(2) {
                Some(x) if i < x => ", ".fmt(f)?,
                Some(x) if i == x => " and ".fmt(f)?,
                _ => (),
            };
        }
        Ok(())
    }
}
