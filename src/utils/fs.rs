use std::{
    fs::{self, File},
    io,
    os::unix,
    path::{Path, PathBuf},
};

use anyhow::Result;
use walkdir::WalkDir;

/// Calls `f` on every path in a directory.
///
/// `skip_root` determines whether the walked directory shouldn't be passed to
/// `f`, and `contents_first` determines whether the directory or it's contents
/// are yielded first.
///
/// The path passed to `f` is absolute.
pub fn walk_dir<P, F, E>(root: P, skip_root: bool, contents_first: bool, f: F) -> Result<(), E>
where
    P: AsRef<Path>,
    F: FnMut(PathBuf) -> Result<(), E>,
{
    let root = root.as_ref();
    WalkDir::new(root)
        .follow_root_links(false)
        .min_depth(skip_root as usize)
        .contents_first(contents_first)
        .into_iter()
        .filter_map(|res| res.map(|entry| entry.into_path()).ok())
        .try_for_each(f)
}

/// Does the same as `walk_dir`, but passes the absolute and relative path to
/// `f` instead of just the absolute path.
pub fn walk_dir_rel<P, F, E>(
    root: P,
    skip_root: bool,
    contents_first: bool,
    mut f: F,
) -> Result<(), E>
where
    P: AsRef<Path>,
    F: FnMut(&Path, &Path) -> Result<(), E>,
{
    let root = root.as_ref();
    walk_dir(root, skip_root, contents_first, |path| {
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
        walk_dir_rel(from, false, false, |path, rel_path| {
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
