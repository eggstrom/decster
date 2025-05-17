use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use bincode::{Decode, Encode};

use crate::packages::PackageManager;

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

    pub fn paths(&self) -> impl Iterator<Item = (&Path, &PathInfo)> {
        self.paths.iter().map(|(path, info)| (path.as_path(), info))
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
}
