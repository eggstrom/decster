use std::{
    fmt::{self, Display, Formatter},
    fs::{self, Permissions},
    os::unix::fs::PermissionsExt,
    path::Path,
    str::FromStr,
};

use anyhow::{Context, Result};
use serde::{
    Deserialize, Deserializer,
    de::{self, Visitor},
};
use thiserror::Error;

use crate::utils::pretty::Pretty;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Mode(u16);

impl Mode {
    pub fn change_mode(&self, path: &Path) -> Result<()> {
        fs::set_permissions(path, Permissions::from_mode(self.0 as u32))
            .with_context(|| format!("Couldn't change mode of {}", path.pretty()))
    }

    const CHARS: [char; 3] = ['r', 'w', 'x'];

    fn parse_char(index: usize, char: char) -> Result<bool, ParseModeError> {
        debug_assert!((0..9).contains(&index));
        Ok(match char {
            '-' => false,
            _ if char == Self::CHARS[index % 3] => true,
            _ => Err(ParseModeError::InvalidChar { char, index })?,
        })
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum ParseModeError {
    #[error("Mode integer isn't in range from 0 to 0o777")]
    IntegerOutOfRange,
    #[error("Mode string isn't 9 characters long")]
    InvalidLength,
    #[error("Invalid mode character `{char}` at index {index}")]
    InvalidChar { char: char, index: usize },
}

impl TryFrom<i64> for Mode {
    type Error = ParseModeError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        (0..=0o777)
            .contains(&value)
            .then_some(Mode(value as u16))
            .ok_or(ParseModeError::IntegerOutOfRange)
    }
}

impl FromStr for Mode {
    type Err = ParseModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 9 {
            Err(ParseModeError::InvalidLength)?;
        }
        let bits = s
            .chars()
            .enumerate()
            .map(|(i, char)| Mode::parse_char(i, char).map(|bit| bit as u16))
            .try_fold(0, |bits, bit| bit.map(|bit| bits << 1 | bit))?;
        Ok(Mode(bits))
    }
}

struct ModeVisitor;

impl Visitor<'_> for ModeVisitor {
    type Value = Mode;

    fn expecting(&self, f: &mut Formatter) -> fmt::Result {
        "an integer or string representing file permissions".fmt(f)
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Mode::try_from(v).map_err(de::Error::custom)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Mode::from_str(v).map_err(de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for Mode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ModeVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_int() {
        let ints = [(-1, false), (0, true), (0o777, true), (0o1000, false)];
        for (int, is_ok) in ints {
            let result = is_ok
                .then(|| Mode::try_from(int).unwrap())
                .ok_or(ParseModeError::IntegerOutOfRange);
            assert_eq!(Mode::try_from(int), result)
        }
    }

    fn invalid_char(char: char, index: usize) -> ParseModeError {
        ParseModeError::InvalidChar { char, index }
    }

    #[test]
    fn parse() {
        let strings = [
            ("---------", Ok(0b000000000)),
            ("r--------", Ok(0b100000000)),
            ("-w-------", Ok(0b010000000)),
            ("--x------", Ok(0b001000000)),
            ("---r-----", Ok(0b000100000)),
            ("----w----", Ok(0b000010000)),
            ("-----x---", Ok(0b000001000)),
            ("------r--", Ok(0b000000100)),
            ("-------w-", Ok(0b000000010)),
            ("--------x", Ok(0b000000001)),
            ("r--r--r--", Ok(0b100100100)),
            ("-w--w--w-", Ok(0b010010010)),
            ("--x--x--x", Ok(0b001001001)),
            ("rwx------", Ok(0b111000000)),
            ("---rwx---", Ok(0b000111000)),
            ("------rwx", Ok(0b000000111)),
            ("rwxrwxrwx", Ok(0b111111111)),
            ("        ", Err(ParseModeError::InvalidLength)),
            ("          ", Err(ParseModeError::InvalidLength)),
            ("         ", Err(invalid_char(' ', 0))),
            (" --------", Err(invalid_char(' ', 0))),
            ("--r------", Err(invalid_char('r', 2))),
            ("w--------", Err(invalid_char('w', 0))),
            ("-x-------", Err(invalid_char('x', 1))),
        ];
        for (s, result) in strings {
            match result {
                Ok(bits) => assert_eq!(Mode::from_str(&s).unwrap(), Mode::try_from(bits).unwrap()),
                Err(err) => assert_eq!(Mode::from_str(&s), Err(err)),
            }
        }
    }
}
