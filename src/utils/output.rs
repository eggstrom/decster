use std::{
    fmt::{self, Display, Formatter},
    path::Path,
};

use crossterm::style::Stylize;

use crate::{paths, state::path::PathKind};

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

pub struct DisplayFile<'a>(&'a Path);

impl Display for DisplayFile<'_> {
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
