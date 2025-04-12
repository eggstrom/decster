use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
    fs, io,
    os::unix::{self, fs::MetadataExt},
    path::Path,
};

use anyhow::{Context, Result, anyhow, bail};
use crossterm::style::Stylize;
use nix::unistd::Uid;

use crate::{
    paths,
    state::{State, path::PathInfo},
    users::Users,
    utils::{self, output::PrettyPathExt, sha256::PathHash},
};

use super::source::ModuleSource;

#[derive(Eq, Ord, PartialEq, PartialOrd)]
enum LinkKind {
    File,
    HardLink,
    Symlink,
}

impl Display for LinkKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            LinkKind::File => "File".green(),
            LinkKind::HardLink => "Hard Link".cyan(),
            LinkKind::Symlink => "Symlink".blue(),
        }
        .fmt(f)
    }
}

#[derive(Eq, Ord, PartialOrd)]
pub struct ModuleLink<'a> {
    kind: LinkKind,
    path: &'a Path,
    source: &'a ModuleSource,
    uid: Option<u32>,
}

impl<'a> ModuleLink<'a> {
    pub fn file(path: &'a Path, source: &'a ModuleSource, uid: Option<Uid>) -> Self {
        ModuleLink {
            kind: LinkKind::File,
            path,
            source,
            uid: uid.map(|uid| uid.as_raw()),
        }
    }

    pub fn hard_link(path: &'a Path, source: &'a ModuleSource, uid: Option<Uid>) -> Self {
        ModuleLink {
            kind: LinkKind::HardLink,
            path,
            source,
            uid: uid.map(|uid| uid.as_raw()),
        }
    }

    pub fn symlink(path: &'a Path, source: &'a ModuleSource, uid: Option<Uid>) -> Self {
        ModuleLink {
            kind: LinkKind::Symlink,
            path,
            source,
            uid: uid.map(|uid| uid.as_raw()),
        }
    }

    pub fn create(&self, users: &mut Users, state: &mut State, module: &str) -> Result<()> {
        let source_path = self.source.fetch(state, module, self.path)?;

        utils::fs::walk_dir_rel(source_path, false, false, |path, rel_path| {
            let mut new_path = paths::untildefy(self.path);
            if rel_path.parent().is_some() {
                new_path = Cow::Owned(new_path.join(rel_path));
            }

            if state.is_path_owned(&new_path) {
                bail!("Path is used by another module");
            } else if path.is_dir() {
                if !new_path.is_dir() {
                    fs::create_dir(&new_path)?;
                    state.add_path(module, &new_path, PathInfo::Directory);
                    self.change_owner(users, &new_path)?;
                }
            } else {
                let info = match self.kind {
                    LinkKind::File => Self::create_file(path, &new_path),
                    LinkKind::HardLink => Self::create_hard_link(path, &new_path),
                    LinkKind::Symlink => Self::create_symlink(path, &new_path),
                }
                .with_context(|| {
                    anyhow!("Couldn't create {} ({})", new_path.pretty(), self.kind)
                })?;
                state.add_path(module, &new_path, info);
                self.change_owner(users, &new_path)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    fn create_file(from: &Path, to: &Path) -> io::Result<PathInfo> {
        let size = from.symlink_metadata()?.size();
        let hash = from.hash_file()?;
        utils::fs::copy(from, to)?;
        Ok(PathInfo::File { size, hash })
    }

    fn create_hard_link(original: &Path, link: &Path) -> io::Result<PathInfo> {
        let size = original.symlink_metadata()?.size();
        let hash = original.hash_file()?;
        fs::hard_link(original, link)?;
        Ok(PathInfo::HardLink { size, hash })
    }

    fn create_symlink(original: &Path, link: &Path) -> io::Result<PathInfo> {
        unix::fs::symlink(original, link)?;
        let original = original.to_path_buf();
        Ok(PathInfo::Symlink { original })
    }

    fn change_owner(&self, users: &mut Users, path: &Path) -> Result<()> {
        if !self.uid.is_some_and(|uid| users.is_current_uid(uid)) {
            unix::fs::lchown(path, self.uid, None)
                .with_context(|| format!("Couldn't change owner of {}", path.pretty()))?;
        }
        Ok(())
    }
}

impl PartialEq for ModuleLink<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Display for ModuleLink<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} -> {}", self.path.pretty(), self.source)
    }
}
