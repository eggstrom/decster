use std::{
    borrow::Cow,
    collections::BTreeSet,
    io::{self, BufRead, Write},
    process::{Command, Stdio},
};

use anyhow::{Context, Result};
use bincode::{Decode, Encode};
use derive_more::From;
use itertools::Itertools;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Decode, Deserialize, Encode, Eq, From, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "kebab-case", expecting = "a supported package manager")]
pub enum PackageManager {
    Pacman,
    Paru,
}

impl PackageManager {
    pub fn sync(
        &self,
        install: bool,
        uninstall: bool,
        packages: &BTreeSet<Cow<str>>,
    ) -> Result<()> {
        let (i, u, p) = (install, uninstall, packages);
        match self {
            PackageManager::Pacman => Self::sync_inner::<Pacman>(i, u, p),
            PackageManager::Paru => Self::sync_inner::<Paru>(i, u, p),
        }
    }

    pub fn sync_inner<M: Manager>(
        install: bool,
        uninstall: bool,
        packages: &BTreeSet<Cow<str>>,
    ) -> Result<()> {
        let installed = Self::list::<M>()?;
        let (old, new) = Self::diff(&installed, packages);
        // Installing comes first so dependencies aren't reinstalled after
        // being uninstalled.
        if install && !new.is_empty() {
            Self::install::<M>(&new)?;
        }
        if uninstall && !old.is_empty() {
            Self::uninstall::<M>(&old)?;
        }
        Ok(())
    }

    fn list<'a, M: Manager>() -> Result<BTreeSet<Cow<'a, str>>> {
        let command = M::LIST;
        Command::new(command[0])
            .args(&command[1..])
            .output()
            .context("Couldn't run command to list installed packages")?
            .stdout
            .lines()
            .map_ok(Cow::Owned)
            .collect::<Result<_, _>>()
            .context("Couldn't parse list of installed packages")
    }

    fn diff<'a>(
        installed: &'a BTreeSet<Cow<str>>,
        wanted: &'a BTreeSet<Cow<str>>,
    ) -> (Vec<&'a str>, Vec<&'a str>) {
        let old = installed.difference(wanted).map(|s| s.as_ref()).collect();
        let new = wanted.difference(installed).map(|s| s.as_ref()).collect();
        (old, new)
    }

    fn update_packages<M: Manager>(args: &[&str], packages: &[&str]) -> io::Result<()> {
        let mut command = Command::new(if M::NEEDS_ROOT { "sudo" } else { args[0] })
            .args(if M::NEEDS_ROOT { args } else { &args[1..] })
            .stdin(Stdio::piped())
            .spawn()?;
        let stdin = command.stdin.as_mut().unwrap();
        for package in packages {
            stdin.write_all(package.as_bytes())?;
            stdin.write_all(b"\n")?;
        }
        command.wait()?;
        Ok(())
    }

    fn install<M: Manager>(packages: &[&str]) -> Result<()> {
        Self::update_packages::<M>(M::INSTALL, packages).context("Couldn't install packages")
    }

    fn uninstall<M: Manager>(packages: &[&str]) -> Result<()> {
        Self::update_packages::<M>(M::UNINSTALL, packages).context("Couldn't uninstall packages")
    }
}

pub trait Manager {
    /// Whether the package manager needs to be run as root.
    const NEEDS_ROOT: bool = true;
    /// Command used to get a list of currently installed packages.
    const LIST: &[&str];
    /// Command used to install packages.
    const INSTALL: &[&str];
    /// Command used to uninstall packages.
    const UNINSTALL: &[&str];
}

struct Pacman;

impl Manager for Pacman {
    const LIST: &[&str] = &["pacman", "-Qqen"];
    const INSTALL: &[&str] = &["pacman", "-S", "-"];
    const UNINSTALL: &[&str] = &["pacman", "-Rnsu", "-"];
}

struct Paru;

impl Manager for Paru {
    const NEEDS_ROOT: bool = false;
    const LIST: &[&str] = &["paru", "-Qqem"];
    const INSTALL: &[&str] = &["paru", "-S", "-"];
    const UNINSTALL: &[&str] = &["paru", "-Rnsu", "-"];
}
