use std::{
    borrow::Cow,
    collections::HashMap,
    fs::{self, File},
    io::Write,
    os::unix::{self, fs::MetadataExt},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use crossterm::style::Stylize;
use derive_more::Display;
use toml::Value;

use crate::{
    fs::{mode::Mode, owner::OwnerIds},
    global::env,
    state::{State, path::PathInfo},
    upon,
    utils::{pretty::Pretty, sha256::Sha256Hash},
};

use super::source::ModuleSource;

#[derive(Clone, Copy, Display, Eq, Ord, PartialEq, PartialOrd)]
pub enum LinkKind {
    #[display("{}", "File".blue())]
    File,
    #[display("{}", "Hard Link".blue())]
    HardLink,
    #[display("{}", "Symlink".blue())]
    Symlink,
    #[display("{}", "Template".blue())]
    Template,
}

#[derive(Display, Eq, Ord, PartialOrd)]
#[display("{} -> {source}", path.pretty())]
pub struct ModuleLink<'a> {
    kind: LinkKind,
    path: &'a Path,
    source: &'a ModuleSource,
    owner: Option<OwnerIds>,
    mode: Option<Mode>,
}

impl<'a> ModuleLink<'a> {
    pub fn new(
        kind: LinkKind,
        path: &'a Path,
        source: &'a ModuleSource,
        owner: Option<OwnerIds>,
        mode: Option<Mode>,
    ) -> Self {
        ModuleLink {
            kind,
            path,
            source,
            owner,
            mode,
        }
    }

    pub fn create(
        &self,
        state: &mut State,
        module: &str,
        context: &HashMap<&str, &Value>,
    ) -> Result<()> {
        let source_path = self.source.fetch(state, module, self.path)?;
        let link_path = env::untildefy(self.path);
        self.create_path(state, module, &link_path)?;

        crate::fs::walk_dir_rel(source_path, false, false, |path, rel_path| {
            let mut new_path = Cow::Borrowed(link_path.as_ref());
            if rel_path.parent().is_some() {
                new_path = Cow::Owned(new_path.join(rel_path));
            }

            if state.is_path_owned(&new_path) {
                bail!("Path is used by another module");
            } else if path.is_dir() {
                self.create_dir(state, module, &new_path)?;
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
                self.change_permissions(&new_path)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    fn create_dir(&self, state: &mut State, module: &str, path: &Path) -> Result<()> {
        if !path.is_dir() {
            fs::create_dir(path)
                .with_context(|| format!("Couldn't create directory: {}", path.pretty()))?;
            state.add_path(module, path, PathInfo::Directory);
            self.change_permissions(path)?;
        }
        Ok(())
    }

    fn create_path(&self, state: &mut State, module: &str, path: &Path) -> Result<()> {
        let mut components = PathBuf::from("");
        if let Some(parent) = path.parent() {
            for component in parent.components() {
                components.push(component);
                self.create_dir(state, module, &components)?;
            }
        }
        Ok(())
    }

    fn create_file(from: &Path, to: &Path) -> Result<PathInfo> {
        let size = from.symlink_metadata()?.size();
        let hash = Sha256Hash::from_file(from)?;
        crate::fs::copy(from, to)?;
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

    fn change_permissions(&self, path: &Path) -> Result<()> {
        if let Some(owner) = &self.owner {
            owner.change_owner(path)?;
        }
        if let Some(mode) = &self.mode {
            mode.change_mode(path)?;
        }
        Ok(())
    }
}

impl PartialEq for ModuleLink<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
