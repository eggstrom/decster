use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
    fs,
    os::unix,
    path::Path,
};

use anyhow::Result;
use crossterm::style::Stylize;
use log::error;
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
                Some(module) => error!(
                    "Couldn't create {} as it's already owned by {}",
                    new_path.pretty(),
                    module.magenta()
                ),
                None => {
                    if let Err(err) = match path.is_dir() {
                        true => self.create_dir(state, &new_path),
                        false => self.create_file(state, path, &new_path),
                    } {
                        error!("Couldn't create {} ({err})", new_path.pretty())
                    }
                }
            }
        });
        Ok(())
    }

    fn create_dir(&self, state: &mut State, path: &Path) -> Result<()> {
        fs::create_dir(path)?;
        state.add_dir(&self.module, path);
        Ok(())
    }

    fn create_file(&self, state: &mut State, from: &Path, to: &Path) -> Result<()> {
        match self.method {
            LinkMethod::Copy => self.create_copy(state, from, to)?,
            LinkMethod::HardLink => self.create_hard_link(state, from, to)?,
            LinkMethod::SoftLink => self.create_soft_link(state, from, to)?,
        }
        Ok(())
    }

    fn create_copy(&self, state: &mut State, from: &Path, to: &Path) -> Result<()> {
        fs::copy(from, to)?;
        state.add_file(&self.module, to)?;
        Ok(())
    }

    fn create_hard_link(&self, state: &mut State, from: &Path, to: &Path) -> Result<()> {
        fs::hard_link(from, to)?;
        state.add_file(&self.module, to)?;
        Ok(())
    }

    fn create_soft_link(&self, state: &mut State, from: &Path, to: &Path) -> Result<()> {
        unix::fs::symlink(from, to)?;
        state.add_link(&self.module, to)?;
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
