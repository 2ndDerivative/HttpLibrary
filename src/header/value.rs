use std::{
    borrow::Borrow,
    fmt::{Display, Formatter, Result as FmtResult},
    ops::Deref,
};

use super::HeaderError;

/// Encodes valid header values that fit the standard requirements:
/// - No empty string
/// - No non-ascii characters
/// - no \r, \n or \0 characters
/// - Removing leading and trailing whitespace
///
///   
/// Can be dereferenced back into a `String` and `&str`
/// Likely to be replaced by a tuple struct.
#[derive(PartialEq, Clone, Debug, Eq)]
pub struct Value {
    value: String,
}
impl Value {
    /// Validates the constraints on strings by the standard.
    pub(crate) fn new<S: AsRef<str>>(s: S) -> Result<Self, HeaderError> {
        let s = s.as_ref().trim();
        if !s.is_ascii() || s.is_empty() || s.contains(['\r', '\n', '\0']) {
            Err(HeaderError)
        } else {
            Ok(Self {
                value: s.to_string(),
            })
        }
    }
    /// Concatenates the current value with a new value with the same key
    /// According to the standard multiple headers like
    /// `head: foo` and `head: bar` are supposed to be parsed like
    /// a single `head: foo,bar`. 
    pub(crate) fn append<S: AsRef<str>>(&mut self, s: S) -> Result<(), HeaderError> {
        let cleaned = Self::new(s)?;
        self.value.push_str(&format!(",{}", cleaned.value));
        Ok(())
    }
}
impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.value)
    }
}
impl Deref for Value {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<S: AsRef<str>> PartialEq<S> for Value {
    fn eq(&self, other: &S) -> bool {
        self.value == other.as_ref()
    }
}
impl PartialEq<str> for Value {
    fn eq(&self, other: &str) -> bool {
        self.value == other
    }
}
impl Borrow<str> for Value {
    fn borrow(&self) -> &str {
        &self.value
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
}
