use std::{
    borrow::Borrow,
    fmt::{Display, Formatter, Result as FmtResult},
};

use super::ValueError;

/// Encodes valid header values that fit the standard requirements:
/// - No empty string
/// - No non-ascii characters
/// - no \r, \n or \0 characters
/// - Removing leading and trailing whitespace
#[derive(PartialEq, Clone, Debug, Eq)]
pub struct Value(String);
impl Value {
    /// Validates the constraints on strings by the standard.
    pub(crate) fn new<S: AsRef<str>>(s: S) -> Result<Self, ValueError> {
        let s = s.as_ref().trim();
        if !s.is_ascii() {
            Err(ValueError::NonAsciiChars)
        } else if s.is_empty() {
            Err(ValueError::EmptyString)
        } else if s.contains(['\r', '\n', '\0']) {
            Err(ValueError::IllegalChars)
        } else {
            Ok(Self(s.to_string()))
        }
    }
    /// Concatenates the current value with a new value with the same key
    /// According to the standard multiple headers like
    /// `head: foo` and `head: bar` are supposed to be parsed like
    /// a single `head: foo,bar`.
    pub(crate) fn append<S: AsRef<str>>(&mut self, s: S) -> Result<(), ValueError> {
        let cleaned = Self::new(s)?;
        self.0.push_str(&format!(",{}", cleaned.0));
        Ok(())
    }
}
impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}
impl<S: AsRef<str>> PartialEq<S> for Value {
    fn eq(&self, other: &S) -> bool {
        self.0 == other.as_ref()
    }
}
impl PartialEq<str> for Value {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}
impl Borrow<str> for Value {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl From<Value> for String {
    fn from(value: Value) -> String {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_null() {
        assert!(Value::new("S\0me text").is_err());
    }
    #[test]
    fn reject_newline() {
        assert!(Value::new("Someo\ne elses problem").is_err());
    }
    #[test]
    fn reject_linefeed() {
        assert!(Value::new("Nobody ca\res").is_err());
    }
    #[test]
    fn reject_multiple_illegal_chars() {
        assert!(Value::new("\n\0body ca\res").is_err());
    }
    #[test]
    fn trim_whitespace() {
        let none = Value::new("some_text");
        let in_front = Value::new("   some_text");
        let trailing = Value::new("some_text      ");
        let both = Value::new("     some_text   ");
        assert_eq!(none, in_front);
        assert_eq!(none, trailing);
        assert_eq!(none, both);
    }
    #[test]
    fn dont_trim_middle_whitespace() {
        let none = Value::new("some_text").unwrap();
        let some = Value::new("some_   text").unwrap();
        assert_ne!(none, some);
    }
    #[test]
    fn dont_ignore_caps() {
        assert_ne!(Value::new("some_text").unwrap(), "SoMe_text");
        assert_eq!(Value::new("some_text").unwrap(), "some_text");
    }
    #[test]
    fn into_string() {
        assert_eq!(Value::new("SOME TEXT").unwrap(), String::from("SOME TEXT"))
    }
}
