use std::{
    fs::{self, File},
    io,
    os::unix::fs::MetadataExt,
    path::Path,
};

use anyhow::Result;
use log::info;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

pub fn dirs_match<P, Q>(lhs: P, rhs: Q) -> Result<bool>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let (lhs, rhs) = (lhs.as_ref(), rhs.as_ref());
    Ok(lhs.is_dir() && rhs.is_dir())
}

pub fn files_match<P, Q>(lhs: P, rhs: Q) -> Result<bool>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let (lhs, rhs) = (lhs.as_ref(), rhs.as_ref());
    Ok(lhs.metadata()?.size() == rhs.metadata()?.size()
        || fs::read_to_string(lhs)? == fs::read_to_string(rhs)?)
}

pub fn hard_links_match<P, Q>(lhs: P, rhs: Q) -> Result<bool>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let (lhs, rhs) = (lhs.as_ref(), rhs.as_ref());
    Ok(lhs.metadata()?.ino() == rhs.metadata()?.ino())
}

pub fn soft_links_match<P, Q>(lhs: P, rhs: Q) -> Result<bool>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let (lhs, rhs) = (lhs.as_ref(), rhs.as_ref());
    Ok(lhs.is_symlink() && rhs.is_symlink() && lhs.read_link()? == rhs.read_link()?)
}

/// Recursively checks whether two paths have the same files.
pub fn all_match<P, Q>(lhs: P, rhs: Q) -> Result<bool>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    for (lhs, rhs) in WalkDir::new(lhs)
        .sort_by_file_name()
        .into_iter()
        .zip(WalkDir::new(rhs).sort_by_file_name())
    {
        let (lhs, rhs) = (lhs?.into_path(), rhs?.into_path());
        let (lhs, rhs) = (lhs.as_path(), rhs.as_path());
        return Ok(dirs_match(lhs, rhs)?
            || files_match(lhs, rhs)?
            || hard_links_match(lhs, rhs)?
            || soft_links_match(lhs, rhs)?);
    }
    Ok(true)
}

pub type Sha256Hash = [u8; 32];

/// Creates a SHA-256 hash from a file's contents.
pub fn hash_file<P>(path: P) -> Result<Sha256Hash>
where
    P: AsRef<Path>,
{
    let mut hasher = Sha256::new();
    io::copy(&mut File::open(path)?, &mut hasher)?;
    Ok(hasher.finalize().into())
}

/// Recursively copies the contents of a directory.
pub fn copy_all<P, Q>(from: P, to: Q) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let (from, to) = (from.as_ref(), to.as_ref());
    info!(
        "Recursively copying `{}` to `{}`",
        from.display(),
        to.display()
    );

    if !from.is_dir() {
        fs::copy(from, to)?;
        return Ok(());
    }

    for path in WalkDir::new(&from) {
        match path {
            Ok(path) => {
                let path = path.path();
                let relative_path = path.strip_prefix(from)?;
                let to = to.join(relative_path);

                if path.is_dir() {
                    fs::create_dir(&to)?;
                } else {
                    fs::copy(path, &to)?;
                }
            }
            Err(error) => Err(error)?,
        }
    }
    Ok(())
}

/// Removes a directory and all it's parents until it finds a parent that's not
/// an empty directory or that it can't delete.
pub fn remove_dir_components<P>(path: P)
where
    P: AsRef<Path>,
{
    let mut path = Some(path.as_ref());
    while let Some(parent) = path {
        match fs::remove_dir(parent) {
            Ok(()) => path = parent.parent(),
            Err(_) => break,
        }
    }
}

/// Removes file or directory recursively.
pub fn remove_all<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    if path.exists() {
        if !path.is_dir() {
            fs::remove_file(path)?;
        } else {
            fs::remove_dir_all(path)?;
        }
    }
    Ok(())
}
