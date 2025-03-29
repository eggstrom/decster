use std::{
    fs::{self, File},
    io,
    os::unix,
    path::{Path, PathBuf},
};

use anyhow::Result;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

/// Call `f` on every path in a directory.
///
/// `contents_first` determines whether the directory or it's contents are
/// yielded first.
pub fn walk_dir<P, F, E>(root: P, contents_first: bool, f: F) -> Result<(), E>
where
    P: AsRef<Path>,
    F: FnMut(PathBuf) -> Result<(), E>,
{
    let root = root.as_ref();
    WalkDir::new(root)
        .follow_root_links(false)
        .contents_first(contents_first)
        .into_iter()
        .filter_map(|res| res.map(|entry| entry.into_path()).ok())
        .try_for_each(f)
}

/// Call `f` on every path in a directory. `f`'s second argument will be the
/// relative path.
///
/// `contents_first` determines whether the directory or it's contents are
/// yielded first.
pub fn walk_dir_with_rel<P, F, E>(root: P, contents_first: bool, mut f: F) -> Result<(), E>
where
    P: AsRef<Path>,
    F: FnMut(&Path, &Path) -> Result<(), E>,
{
    let root = root.as_ref();
    walk_dir(root, contents_first, |path| {
        f(
            path.as_path(),
            path.strip_prefix(root)
                .expect("Paths should always be prefixed with the root while walking a directory"),
        )
    })
}

/// Copies a file without following symlinks.
pub fn copy<P, Q>(from: P, to: Q) -> io::Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let (from, to) = (from.as_ref(), to.as_ref());
    if from.is_symlink() {
        unix::fs::symlink(from.read_link()?, to)?;
    } else {
        // `fs::copy` isn't used because it overwrites files.
        io::copy(&mut File::open(from)?, &mut File::create_new(to)?)?;
    }
    Ok(())
}

/// Recursively copies the contents of a directory.
pub fn copy_all<P, Q>(from: P, to: Q) -> io::Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let (from, to) = (from.as_ref(), to.as_ref());
    if !from.is_dir() {
        copy(from, to)
    } else {
        walk_dir_with_rel(from, false, |path, rel_path| {
            let to = to.join(rel_path);
            if path.is_dir() {
                fs::create_dir(to)
            } else {
                copy(path, to)
            }
        })
    }
}

/// Recursively removes a file or directory.
pub fn remove_all<P>(path: P) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

pub type Sha256Hash = [u8; 32];

/// Creates a SHA-256 hash from a file's contents.
pub fn hash_file<P>(path: P) -> io::Result<Sha256Hash>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    Ok(hasher.finalize().into())
}
