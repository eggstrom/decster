use std::{
    fmt::{self, Display, Formatter},
    path::Path,
};

use crossterm::style::Stylize;

pub trait Pretty {
    type Target<'a>: Display
    where
        Self: 'a;

    fn pretty(&self) -> Self::Target<'_>;
}

pub struct PrettyPath<'a>(&'a Path);

impl Display for PrettyPath<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(file) = self.0.file_name() {
            if let Some(path) = self.0.parent() {
                write!(f, "{}/", path.display())?;
            }
            file.to_string_lossy().magenta().fmt(f)?;
        }
        Ok(())
    }
}

impl<T> Pretty for T
where
    T: AsRef<Path>,
{
    type Target<'a>
        = PrettyPath<'a>
    where
        Self: 'a;

    fn pretty(&self) -> Self::Target<'_> {
        PrettyPath(self.as_ref())
    }
}
