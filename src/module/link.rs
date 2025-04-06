use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
    fs, io,
    os::unix::{self, fs::MetadataExt},
    path::Path,
};

use crate::{
    out, paths,
    state::{
        State,
        path::{PathInfo, PathKind},
    },
    utils::{self, output::PathDisplay, sha256::PathHash},
};

use super::source::ModuleSource;

pub struct ModuleLink<'a> {
    path: &'a Path,
    source: &'a ModuleSource,
}

impl<'a> ModuleLink<'a> {
    pub fn new(path: &'a Path, source: &'a ModuleSource) -> Self {
        ModuleLink { path, source }
    }

    fn create_with<F>(&self, state: &mut State, module: &str, mut f: F)
    where
        F: FnMut(&mut State, &Path, &Path) -> io::Result<()>,
    {
        let source_path = match self.source.path(module, self.path) {
            Ok(path) => path,
            Err(err) => {
                out!(2, R; "{err}");
                return;
            }
        };

        let _ = utils::fs::walk_dir_rel(source_path, false, false, |path, rel_path| {
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
                out!(2, R; "{}", new_path.display_kind(kind); "Path is used");
            } else if let PathKind::Directory = kind {
                state.create_dir(module, &new_path);
            } else if let Err(err) = f(state, path, &new_path) {
                out!(2, R; "{}", new_path.display_file(); "{err}");
            } else {
                out!(2, G; "{}", new_path.display_file());
            }
            Ok::<_, ()>(())
        });
    }

    pub fn create_files(&self, state: &mut State, module: &str) {
        self.create_with(state, module, |state, from, to| {
            let size = from.symlink_metadata()?.size();
            let hash = from.hash_file()?;
            utils::fs::copy(from, to)?;
            state.add_path(module, to, PathInfo::File { size, hash });
            Ok(())
        });
    }

    pub fn create_hard_links(&self, state: &mut State, module: &str) {
        self.create_with(state, module, |state, original, link| {
            let size = original.symlink_metadata()?.size();
            let hash = original.hash_file()?;
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
        write!(f, "{} -> {}", self.path.display_file(), self.source)
    }
}
