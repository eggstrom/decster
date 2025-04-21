use std::{
    borrow::Cow,
    collections::HashMap,
    fs::{self, File},
    io::Write,
    os::unix::{self, fs::MetadataExt},
    path::Path,
    rc::Rc,
};

use anyhow::{Context, Result, bail};
use crossterm::style::Stylize;
use derive_more::Display;
use toml::Value;

use crate::{
    global::env::{self, User},
    state::{State, path::PathInfo},
    upon,
    utils::{self, pretty::Pretty, sha256::Sha256Hash},
};

use super::source::ModuleSource;

#[derive(Display, Eq, Ord, PartialEq, PartialOrd)]
enum LinkKind {
    #[display("{}", "File".blue())]
    File,
    #[display("{}", "Hard Link".blue())]
    HardLink,
    #[display("{}", "Symlink".blue())]
    Symlink,
    #[display("{}", "Template".blue())]
    Template,
}

#[derive(Eq, Ord, PartialOrd)]
pub struct ModuleLink<'a> {
    kind: LinkKind,
    path: &'a Path,
    source: &'a ModuleSource,
    user: Option<Rc<User>>,
}

impl<'a> ModuleLink<'a> {
    fn new(
        kind: LinkKind,
        path: &'a Path,
        source: &'a ModuleSource,
        user: Option<&Rc<User>>,
    ) -> Self {
        ModuleLink {
            kind,
            path,
            source,
            user: user.map(Rc::clone),
        }
    }

    pub fn file(path: &'a Path, source: &'a ModuleSource, user: Option<&Rc<User>>) -> Self {
        ModuleLink::new(LinkKind::File, path, source, user)
    }

    pub fn hard_link(path: &'a Path, source: &'a ModuleSource, user: Option<&Rc<User>>) -> Self {
        ModuleLink::new(LinkKind::HardLink, path, source, user)
    }

    pub fn symlink(path: &'a Path, source: &'a ModuleSource, user: Option<&Rc<User>>) -> Self {
        ModuleLink::new(LinkKind::Symlink, path, source, user)
    }

    pub fn template(path: &'a Path, source: &'a ModuleSource, user: Option<&Rc<User>>) -> Self {
        ModuleLink::new(LinkKind::Template, path, source, user)
    }

    pub fn create(
        &self,
        state: &mut State,
        module: &str,
        context: &HashMap<&str, &Value>,
    ) -> Result<()> {
        let source_path = self.source.fetch(state, module, self.path)?;

        utils::fs::walk_dir_rel(source_path, false, false, |path, rel_path| {
            let mut new_path = match &self.user {
                Some(user) => user.untildefy(self.path),
                None => env::untildefy(self.path),
            };
            if rel_path.parent().is_some() {
                new_path = Cow::Owned(new_path.join(rel_path));
            }

            if state.is_path_owned(&new_path) {
                bail!("Path is used by another module");
            } else if path.is_dir() {
                if !new_path.is_dir() {
                    fs::create_dir(&new_path)?;
                    state.add_path(module, &new_path, PathInfo::Directory);
                    self.change_owner(&new_path)?;
                }
            } else {
                let info = match self.kind {
                    LinkKind::File => Self::create_file(path, &new_path),
                    LinkKind::HardLink => Self::create_hard_link(path, &new_path),
                    LinkKind::Symlink => Self::create_symlink(path, &new_path),
                    LinkKind::Template => Self::create_template(path, &new_path, context),
                }
                .with_context(|| {
                    let new_path = env::tildefy(new_path.as_ref());
                    format!("Couldn't create {} ({})", new_path.pretty(), self.kind)
                })?;
                state.add_path(module, &new_path, info);
                self.change_owner(&new_path)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    fn create_file(from: &Path, to: &Path) -> Result<PathInfo> {
        let size = from.symlink_metadata()?.size();
        let hash = Sha256Hash::from_file(from)?;
        utils::fs::copy(from, to)?;
        Ok(PathInfo::File { size, hash })
    }

    fn create_hard_link(original: &Path, link: &Path) -> Result<PathInfo> {
        let size = original.symlink_metadata()?.size();
        let hash = Sha256Hash::from_file(original)?;
        fs::hard_link(original, link)?;
        Ok(PathInfo::HardLink { size, hash })
    }

    fn create_symlink(original: &Path, link: &Path) -> Result<PathInfo> {
        unix::fs::symlink(original, link)?;
        let original = original.to_path_buf();
        Ok(PathInfo::Symlink { original })
    }

    fn create_template(
        from: &Path,
        to: &Path,
        context: &HashMap<&str, &Value>,
    ) -> Result<PathInfo> {
        let template = fs::read_to_string(from)?;
        let render = upon::render(&template, context)?;
        File::create_new(to)?.write_all(render.as_bytes())?;
        let size = render.len() as u64;
        let hash = Sha256Hash::from_bytes(render);
        Ok(PathInfo::File { size, hash })
    }

    fn change_owner(&self, path: &Path) -> Result<()> {
        if let Some(user) = &self.user {
            user.change_owner(path)?;
        }
        Ok(())
    }
}

impl PartialEq for ModuleLink<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
