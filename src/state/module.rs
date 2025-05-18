use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use bincode::{Decode, Encode};
use crossterm::style::Stylize;
use termtree::Tree;

use crate::{globs::Globs, packages::PackageManager, utils::pretty::Pretty};

use super::path::PathInfo;

#[derive(Decode, Encode)]
pub struct ModuleState {
    paths: Vec<(PathBuf, PathInfo)>,
    packages: BTreeMap<PackageManager, BTreeSet<String>>,
}

impl ModuleState {
    pub fn with_packages(packages: BTreeMap<PackageManager, BTreeSet<String>>) -> Self {
        let paths = Vec::new();
        Self { paths, packages }
    }

    pub fn packages(&self) -> impl Iterator<Item = (PackageManager, impl Iterator<Item = &str>)> {
        self.packages
            .iter()
            .map(|(m, p)| (*m, p.iter().map(|s| s.as_str())))
    }

    pub fn paths_mut(&mut self) -> &mut Vec<(PathBuf, PathInfo)> {
        &mut self.paths
    }

    pub fn push_path(&mut self, path: PathBuf, info: PathInfo) {
        self.paths.push((path, info));
    }

    pub fn clear_packages(&mut self) {
        self.packages.clear();
    }

    pub fn tree<'a>(&'a self, name: &'a str, globs: &Globs) -> Tree<Cow<'a, str>> {
        let paths = self
            .paths
            .iter()
            .filter(|(path, _)| globs.is_match(path))
            .map(|(path, info)| Cow::Owned(format!("{} ({})", path.pretty(), info.kind())));
        let packages = self.packages.iter().map(|(manager, packages)| {
            Tree::new(manager.to_string().into()).with_leaves(
                packages
                    .iter()
                    .filter(|package| globs.is_match(package))
                    .map(|s| Cow::Owned(s.as_str().magenta().to_string())),
            )
        });
        Tree::new(name.magenta().to_string().into()).with_leaves([
            Tree::new("Paths".into()).with_leaves(paths),
            Tree::new("Packages".into()).with_leaves(packages),
        ])
    }
}
