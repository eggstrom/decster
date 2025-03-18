use std::{fs, path::Path};

use anyhow::Result;
use walkdir::WalkDir;

pub fn remove_all<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    if path.exists() {
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

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
