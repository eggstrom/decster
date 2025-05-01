use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    io,
    path::Path,
    str::FromStr,
};

use anyhow::Result;
use bincode::{Decode, Encode};
use hex::{FromHex, FromHexError};
use serde::{Deserialize, Deserializer, de};
use sha2::{Digest, Sha256};

#[derive(Clone, Decode, Encode, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Sha256Hash([u8; 32]);

impl Sha256Hash {
    pub fn from_bytes<B>(bytes: B) -> Sha256Hash
    where
        B: AsRef<[u8]>,
    {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hasher.finalize().into()
    }

    /// Creates a SHA-256 hash from a file's contents.
    pub fn from_file(path: &Path) -> io::Result<Sha256Hash> {
        let mut hasher = Sha256::new();
        io::copy(&mut File::open(path)?, &mut hasher)?;
        Ok(hasher.finalize().into())
    }

    /// Creates a SHA-256 hash from a symlink.
    pub fn from_symlink(path: &Path) -> io::Result<Sha256Hash> {
        Ok(Sha256::digest(path.read_link()?.to_string_lossy().as_ref()).into())
    }

    /// Creates a SHA-256 hash from a path's contents.
    pub fn from_path(path: &Path) -> Result<Sha256Hash> {
        if path.is_symlink() {
            return Ok(Self::from_symlink(path)?);
        } else if path.is_file() {
            return Ok(Self::from_file(path)?);
        }

        let mut hasher = Sha256::new();
        crate::fs::walk_dir_rel(path, true, false, |path, rel_path| {
            hasher.update(rel_path.to_string_lossy().as_ref());
            if path.is_symlink() {
                hasher.update(path.read_link()?.to_string_lossy().as_ref());
            } else if path.is_file() {
                io::copy(&mut File::open(path)?, &mut hasher)?;
            }
            Ok(())
        })?;
        Ok(hasher.finalize().into())
    }
}

impl<T> From<T> for Sha256Hash
where
    T: Into<[u8; 32]>,
{
    fn from(value: T) -> Self {
        Sha256Hash(value.into())
    }
}

impl FromStr for Sha256Hash {
    type Err = FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Sha256Hash(<[u8; 32]>::from_hex(s)?))
    }
}

impl Display for Sha256Hash {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02X}")?;
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for Sha256Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Sha256Hash::from_str(&String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use itertools::izip;

    use super::*;

    #[test]
    fn parse_and_display() {
        let strings = [
            "CA978112CA1BBDCAFAC231B39A23DC4DA786EFF8147C4E72B9807785AFEE48BB",
            "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb",
            "CA978112CA1BBDCAFAC231B39A23DC4da786eff8147c4e72b9807785afee48bb",
            "CA978112CA1BBDCAFAC231B39A23DC4DA786EFF8147C4E72B9807785AFEE48B",
        ];
        let hashes: [Result<Sha256Hash, _>; 4] = strings.map(|s| s.parse());
        let is_ok = [true, true, true, false];
        for (string, hash, is_ok) in izip!(strings, hashes, is_ok) {
            assert_eq!(hash.is_ok(), is_ok);
            if is_ok {
                assert_eq!(hash.unwrap().to_string(), string.to_uppercase());
            }
        }
    }
}
