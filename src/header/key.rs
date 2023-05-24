use std::{
    borrow::Borrow,
    fmt::{Display, Formatter, Result as FmtResult},
};

use super::KeyError;

#[derive(PartialEq, Hash, Debug, Eq, Clone)]
/// Struct with all requirements encoded.
/// Always stores as its ascii lowercase item.
/// - Can't contain the empty string.
/// - Equals with any case of the same characters.
/// - cannot have leading or trailing whitespace
pub struct Key(String);
impl Key {
    /// Verifies compliance with the HTTP/1.1 header
    /// standard, ensuring that [Key] always matches it.
    pub fn new<S: AsRef<str>>(s: S) -> Result<Self, KeyError> {
        let s = s.as_ref();
        if !s.is_ascii() {
            Err(KeyError::NonAsciiChars)
        } else if s.is_empty() {
            Err(KeyError::EmptyString)
        } else if s.trim_start() != s {
            Err(KeyError::LeadingWhitespace)
        } else if s.trim_end() != s {
            Err(KeyError::ColonWhitespace)
        } else {
            Ok(Self(s.to_ascii_lowercase()))
        }
    }
}
impl Display for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}
impl<S: AsRef<str>> PartialEq<S> for Key {
    fn eq(&self, other: &S) -> bool {
        self.0 == other.as_ref().to_ascii_lowercase()
    }
}
impl Borrow<str> for Key {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl From<Key> for String {
    fn from(value: Key) -> String {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn eq_string_ignore_case() {
        let result = Key::new("AbC").unwrap();
        assert_eq!(result, "abc");
        assert_eq!(result, "ABC");
    }
    #[test]
    fn eq_string_refuse_whitespace() {
        assert_ne!(Key::new("AbC").unwrap(), "  ABC");
    }
    #[test]
    fn eq_self_ignore_case() {
        assert_eq!(Key::new("ABC"), Key::new("abc"));
    }
    #[test]
    fn refuse_whitespace() {
        assert!(Key::new("      abc         ").is_err())
    }
    #[test]
    fn refuse_whitespace_trailing() {
        assert!(Key::new("abc ").is_err());
    }
    #[test]
    fn refuse_whitespace_leading() {
        assert!(Key::new(" abc").is_err())
    }
}
