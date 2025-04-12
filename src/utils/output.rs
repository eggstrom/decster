use std::{
    fmt::{self, Display, Formatter},
    path::Path,
};

use crossterm::style::Stylize;

use crate::paths;

#[macro_export]
macro_rules! out_inner {
    ($indent:expr $(, $color:ident)?; $($msg:expr),+ $(; $($info:expr),+)?) => {{
        #[allow(unused_imports)]
        use crossterm::{style::{Color, SetForegroundColor, Stylize}};

        if !$crate::config::quiet() {
            $crate::out_inner!(indent $indent);
            $crate::out_inner!(label $($color)?);
            print!($($msg),+);
            $crate::out_inner!(info $($color)? $(; $($info),+)?);
            println!();
        }
    }};

    (indent 0) => {};
    (indent $indent:expr) => { (0_usize..$indent).for_each(|_| print!("  ")); };

    (label) => {};
    (label Green) => { print!("{} ", "Success:".green()); };
    (label Yellow) => { print!("{} ", "Skipped:".yellow()); };
    (label Red) => { print!("{} ", "Failure:".red()); };

    (info) => {};
    (info $color:ident) => {};
    (info $($info:expr),+) => {
        print!(" (");
        print!($($info),+);
        print!(")");
    };
    (info $color:ident; $($info:expr),+) => {
        print!(" {}(", SetForegroundColor(Color::$color));
        print!($($info),+);
        print!("){}", SetForegroundColor(Color::Reset));
    };
}

#[macro_export]
macro_rules! out {
    ($indent:expr;    $($msg:expr),+ $(; $($info:expr),+)?) => { $crate::out_inner!($indent;         $($msg),+ $(; $($info),+)?) };
    ($indent:expr, G; $($msg:expr),+ $(; $($info:expr),+)?) => { $crate::out_inner!($indent, Green;  $($msg),+ $(; $($info),+)?) };
    ($indent:expr, Y; $($msg:expr),+ $(; $($info:expr),+)?) => { $crate::out_inner!($indent, Yellow; $($msg),+ $(; $($info),+)?) };
    ($indent:expr, R; $($msg:expr),+ $(; $($info:expr),+)?) => { $crate::out_inner!($indent, Red;    $($msg),+ $(; $($info),+)?) };
}

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
