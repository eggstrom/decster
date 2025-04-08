use globset::{GlobBuilder, GlobSet};

pub trait GlobSetExt {
    fn from_globs<I>(iter: I) -> Result<GlobSet, globset::Error>
    where
        I: IntoIterator,
        I::Item: AsRef<str>;
}

impl GlobSetExt for GlobSet {
    fn from_globs<I>(iter: I) -> Result<GlobSet, globset::Error>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let mut globs = GlobSet::builder();
        for glob in iter.into_iter() {
            let glob = GlobBuilder::new(glob.as_ref())
                .literal_separator(true)
                .build()?;
            globs.add(glob);
        }
        globs.build()
    }
}
