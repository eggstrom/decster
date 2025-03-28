use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
    fs::{self, File},
    io,
    os::unix::{self, fs::MetadataExt},
    path::Path,
};

use crossterm::style::Stylize;

use crate::{
    config, out, paths,
    source::path::SourcePath,
    state::{
        State,
        path::{PathInfo, PathKind},
    },
    utils::{self, output::PathExt},
};

pub struct ModuleLink<'a> {
    path: &'a Path,
    source: &'a SourcePath,
}

impl<'a> ModuleLink<'a> {
    pub fn new(path: &'a Path, source: &'a SourcePath) -> Self {
        ModuleLink { path, source }
    }

    fn create_with<F>(&self, state: &mut State, name: &str, mut f: F)
    where
        F: FnMut(&mut State, &Path, &Path) -> io::Result<()>,
    {
        let _: Result<_, ()> =
            utils::fs::walk_dir_with_rel(self.source.path(), false, |path, rel_path| {
                let mut new_path = paths::untildefy(self.path);
                if rel_path.parent().is_some() {
                    new_path = Cow::Owned(new_path.join(rel_path));
                }

                let kind = if path.is_dir() {
                    PathKind::Directory
                } else {
                    PathKind::File
                };

                if state.has_path(&new_path) {
                    out!(2, failed, "{} (Path is used)", new_path.display_kind(kind));
                } else if let Err(err) = if let PathKind::Directory = kind {
                    state.create_dir(name, &new_path)
                } else {
                    f(state, path, &new_path)
                } {
                    out!(2, failed, "{} ({err})", new_path.display_kind(kind));
                } else {
                    out!(2, created, "{}", new_path.display_kind(kind));
                }
                Ok(())
            });
    }

    pub fn create_files(&self, state: &mut State, module: &str) {
        self.create_with(state, module, |state, from, to| {
            let size = from.symlink_metadata()?.size();
            let hash = utils::fs::hash_file(from)?;
            io::copy(&mut File::open(from)?, &mut File::create_new(to)?)?;
            state.add_path(module, to, PathInfo::File { size, hash });
            Ok(())
        });
    }

    pub fn create_hard_links(&self, state: &mut State, module: &str) {
        self.create_with(state, module, |state, original, link| {
            let size = original.symlink_metadata()?.size();
            let hash = utils::fs::hash_file(original)?;
            fs::hard_link(original, link)?;
            state.add_path(module, link, PathInfo::HardLink { size, hash });
            Ok(())
        });
    }

    pub fn create_symlinks(&self, state: &mut State, module: &str) {
        self.create_with(state, module, |state, original, link| {
            unix::fs::symlink(original, link)?;
            let original = original.to_path_buf();
            state.add_path(module, link, PathInfo::Symlink { original });
            Ok(())
        });
    }
}

impl Display for ModuleLink<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} -> {}", self.path.display_file(), self.source.name)?;
        if let Some(source_path) = &self.source.path {
            source_path.display_file().fmt(f)?;
        }
        Ok(())
    }
}
