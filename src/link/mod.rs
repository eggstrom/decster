use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
    fs, io,
    os::unix,
    path::Path,
};

use anyhow::Result;
use crossterm::style::Stylize;
use method::LinkMethod;

use crate::{
    source::path::SourcePath,
    state::State,
    utils::{self, output::Pretty},
};

pub mod method;

pub struct Link<'a> {
    pub module: &'a str,
    pub path: &'a Path,
    pub source: &'a SourcePath,
    pub method: LinkMethod,
}

impl<'a> Link<'a> {
    pub fn new(
        module: &'a str,
        path: &'a Path,
        source: &'a SourcePath,
        method: LinkMethod,
    ) -> Self {
        Link {
            module,
            path,
            source,
            method,
        }
    }

    pub fn enable(&self, state: &mut State) -> Result<()> {
        utils::fs::walk_dir_with_rel(self.source.path()?, false, |path, rel_path| {
            let new_path = match rel_path.parent() {
                None => Cow::Borrowed(self.path),
                Some(_) => Cow::Owned(self.path.join(rel_path)),
            };

            match state.owner(&new_path) {
                Some(module) => println!(
                    "{} {} (Path is owned by {})",
                    "Failed:".red(),
                    new_path.pretty(),
                    module.magenta()
                ),
                None => {
                    if let Err(error) = match path.is_dir() {
                        true => self.create_dir(state, &new_path),
                        false => self.create_file(state, path, &new_path),
                    } {
                        println!("{} {} ({error})", "Failed:".red(), new_path.pretty())
                    } else {
                        println!("  {} {}", "Created:".green(), path.pretty());
                    }
                }
            }
        });
        Ok(())
    }

    fn create_dir(&self, state: &mut State, path: &Path) -> io::Result<()> {
        if !path.is_dir() {
            fs::create_dir(path)?;
            state.add_dir(&self.module, path);
        }
        Ok(())
    }

    fn create_file(&self, state: &mut State, source: &Path, destination: &Path) -> io::Result<()> {
        match self.method {
            LinkMethod::Copy => self.create_copy(state, source, destination)?,
            LinkMethod::HardLink => self.create_hard_link(state, source, destination)?,
            LinkMethod::SoftLink => self.create_soft_link(state, source, destination)?,
        }
        Ok(())
    }

    fn create_copy(&self, state: &mut State, from: &Path, to: &Path) -> io::Result<()> {
        fs::copy(from, to)?;
        state.add_file(&self.module, to);
        Ok(())
    }

    fn create_hard_link(&self, state: &mut State, original: &Path, link: &Path) -> io::Result<()> {
        fs::hard_link(original, link)?;
        state.add_file(&self.module, link);
        Ok(())
    }

    fn create_soft_link(&self, state: &mut State, original: &Path, link: &Path) -> io::Result<()> {
        unix::fs::symlink(original, link)?;
        state.add_link(&self.module, original, link);
        Ok(())
    }
}

impl Display for Link<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.path.pretty(),
            "->".with(self.method.color()),
            self.source,
            self.method
        )
    }
}
