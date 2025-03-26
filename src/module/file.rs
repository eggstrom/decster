use std::{borrow::Cow, fs, io, os::unix, path::Path};

use crossterm::style::Stylize;

use crate::{
    source::path::SourcePath,
    state::State,
    utils::{self, output::Pretty},
};

pub struct ModuleFile<'a> {
    path: &'a Path,
    source: &'a SourcePath,
}

impl<'a> ModuleFile<'a> {
    pub fn new(path: &'a Path, source: &'a SourcePath) -> Self {
        ModuleFile { path, source }
    }

    fn create_with<F>(&self, state: &mut State, name: &str, mut f: F)
    where
        F: FnMut(&mut State, &Path, &Path) -> io::Result<()>,
    {
        let _ = utils::fs::walk_dir_with_rel(self.source.path(), false, |path, rel_path| {
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
                    if let Err(err) = match path.is_dir() {
                        true => state.create_dir(name, &new_path),
                        false => f(state, path, &new_path),
                    } {
                        println!("    {} {} ({err})", "Failed:".red(), new_path.pretty())
                    } else {
                        println!("    {} {}", "Created:".green(), new_path.pretty());
                    }
                }
            }
            Ok(())
        });
    }

    pub fn create_files(&self, state: &mut State, name: &str) {
        self.create_with(state, name, |state, from, to| {
            fs::copy(from, to)?;
            state.add_file(name, to);
            Ok(())
        });
    }

    pub fn create_hard_links(&self, state: &mut State, name: &str) {
        self.create_with(state, name, |state, original, link| {
            fs::hard_link(original, link)?;
            state.add_hard_link(name, link);
            Ok(())
        });
    }

    pub fn create_symlinks(&self, state: &mut State, name: &str) {
        self.create_with(state, name, |state, original, link| {
            unix::fs::symlink(original, link)?;
            state.add_symlink(name, original, link);
            Ok(())
        });
    }
}
