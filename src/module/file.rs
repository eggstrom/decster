use std::{
    borrow::Cow,
    fs::{self, File},
    io,
    os::unix::{self, fs::MetadataExt},
    path::Path,
};

use crossterm::style::Stylize;

use crate::{
    global::paths,
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
            let mut new_path = paths::untildefy(self.path);
            if let Some(_) = rel_path.parent() {
                new_path = Cow::Owned(new_path.join(rel_path));
            }

            if state.is_path_used(&new_path) {
                println!("{} {} (Path is in use)", "Failed:".red(), new_path.pretty())
            } else {
                if let Err(err) = if path.is_dir() {
                    state.create_dir(name, &new_path)
                } else {
                    f(state, path, &new_path)
                } {
                    println!("    {} {} ({err})", "Failed:".red(), new_path.pretty())
                } else {
                    println!("    {} {}", "Created:".green(), new_path.pretty());
                }
            }
            Ok(())
        });
    }

    pub fn create_files(&self, state: &mut State, module: &str) {
        self.create_with(state, module, |state, from, to| {
            let size = from.metadata()?.size();
            let hash = utils::fs::hash_file(from)?;
            io::copy(&mut File::open(from)?, &mut File::create_new(to)?)?;
            state.add_file(module, to, size, hash);
            Ok(())
        });
    }

    pub fn create_hard_links(&self, state: &mut State, module: &str) {
        self.create_with(state, module, |state, original, link| {
            let size = original.metadata()?.size();
            let hash = utils::fs::hash_file(original)?;
            fs::hard_link(original, link)?;
            state.add_hard_link(module, link, size, hash);
            Ok(())
        });
    }

    pub fn create_symlinks(&self, state: &mut State, module: &str) {
        self.create_with(state, module, |state, original, link| {
            unix::fs::symlink(original, link)?;
            state.add_symlink(module, original, link);
            Ok(())
        });
    }
}
