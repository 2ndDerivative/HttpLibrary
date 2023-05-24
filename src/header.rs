use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

pub mod key;
pub mod value;

pub use key::Key;
pub use value::Value;

#[derive(PartialEq, Debug)]
pub enum HeaderError {
    Key(KeyError),
    Value(ValueError),
    NoSeparator,
}
impl Error for HeaderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Key(e) => Some(e),
            Self::Value(e) => Some(e),
            Self::NoSeparator => None,
        }
    }
}
impl Display for HeaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let (v, error) = match self {
            Self::Key(e) => ("Key", e.to_string()),
            Self::Value(e) => ("Value", e.to_string()),
            Self::NoSeparator => (
                "Header",
                "missing key-value pair separated by ': '".to_string(),
            ),
        };
        write!(f, "{v}: {error}")
    }
}

impl From<KeyError> for HeaderError {
    fn from(value: KeyError) -> Self {
        Self::Key(value)
    }
}

impl From<ValueError> for HeaderError {
    fn from(value: ValueError) -> Self {
        Self::Value(value)
    }
}

#[derive(PartialEq, Debug)]
pub enum KeyError {
    NonAsciiChars,
    EmptyString,
    LeadingWhitespace,
    // Strong security risk!
    ColonWhitespace,
}
impl Error for KeyError {}
impl Display for KeyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}",
            match self {
                Self::NonAsciiChars => "non-ascii chars",
                Self::EmptyString => "empty key",
                Self::LeadingWhitespace => "leading whitespace",
                Self::ColonWhitespace => "pre-colon whitespace",
            }
        )
    }
}

#[derive(PartialEq, Debug)]
pub enum ValueError {
    NonAsciiChars,
    EmptyString,
    IllegalChars,
}
impl Error for ValueError {}
impl Display for ValueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}",
            match self {
                Self::NonAsciiChars => "non-ascii chars",
                Self::EmptyString => "empty value",
                Self::IllegalChars => "illegal characters (\\r, \\n or \\0)",
            }
        )
    }
}
