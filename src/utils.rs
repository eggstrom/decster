use std::{fs, path::Path};

use anyhow::{Context, Result};
use walkdir::WalkDir;

/// Removes file or directory.
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

/// Removes a directory and all it's parents until it finds a parent that's not
/// empty or that it can't delete.
pub fn remove_dir_all<P>(path: P)
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

/// Recursively checks whether two paths have the same contents.
pub fn all_files_match<P, Q>(lhs: P, rhs: Q) -> Result<bool>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let (lhs, rhs) = (lhs.as_ref(), rhs.as_ref());
    for (lhs, rhs) in WalkDir::new(lhs)
        .sort_by_file_name()
        .into_iter()
        .zip(WalkDir::new(rhs).sort_by_file_name())
    {
        let (lhs, rhs) = (lhs?.into_path(), rhs?.into_path());

        if lhs.is_dir() && rhs.is_dir() {
            continue;
        } else if lhs.is_symlink() && rhs.is_symlink() && lhs.read_link()? == rhs.read_link()? {
            continue;
        } else if fs::read_to_string(lhs)? == fs::read_to_string(rhs)? {
            continue;
        }
        return Ok(false);
    }
    Ok(true)
}

/// Recursively copies the contents of a directory.
pub fn copy_all<P, Q>(from: P, to: Q) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let (from, to) = (from.as_ref(), to.as_ref());
    if !from.is_dir() {
        println!("  {} -> {}", from.display(), to.display());
        fs::copy(from, to)?;
        return Ok(());
    }

    for path in WalkDir::new(&from) {
        match path {
            Ok(path) => {
                let path = path.path();
                let relative_path = path.strip_prefix(from)?;
                let to = to.join(relative_path);

                println!("  {} -> {}", path.display(), to.display());

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
