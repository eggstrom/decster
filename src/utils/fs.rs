use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use walkdir::WalkDir;

/// Call `f` on every path in a directory.
///
/// `contents_first` determines whether the directory or it's contents are
/// yielded first.
pub fn walk_dir<P, F>(root: P, contents_first: bool, f: F) -> io::Result<()>
where
    P: AsRef<Path>,
    F: FnMut(PathBuf) -> io::Result<()>,
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
pub fn walk_dir_with_rel<P, F>(root: P, contents_first: bool, mut f: F) -> io::Result<()>
where
    P: AsRef<Path>,
    F: FnMut(&Path, &Path) -> io::Result<()>,
{
    let root = root.as_ref();
    walk_dir(root, contents_first, |path| {
        f(
            path.as_path(),
            path.strip_prefix(root)
                .expect("paths should always be prefixed with the root while walking a directory"),
        )
    })
}

/// Recursively copies the contents of a directory.
pub fn copy_all<P, Q>(from: P, to: Q) -> io::Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let (from, to) = (from.as_ref(), to.as_ref());
    if !from.is_dir() {
        fs::copy(from, to)?;
        return Ok(());
    }

    walk_dir_with_rel(from, false, |path, rel_path| {
        let to = to.join(rel_path);
        if path.is_dir() {
            fs::create_dir(&to)?;
        } else {
            fs::copy(path, &to)?;
        }
        Ok(())
    })
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

/// Recursively removes `path` if it exists.
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
