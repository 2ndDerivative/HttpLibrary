use std::{
    borrow::Borrow,
    fmt::{Display, Formatter, Result as FmtResult},
    ops::Deref,
};

use super::HeaderError;

#[derive(PartialEq, Debug, Hash, Eq, Clone)]
/// Struct with all requirements encoded.
/// Always stores as its ascii lowercase item.
/// Can't contain the empty string.
/// Equals with any case of the same characters.
///  
/// Can be dereferenced back into a `String` and `&str`
/// Likely to be replaced by a tuple struct.
pub struct Key {
    value: String,
}
impl Key {
    /// Verifies compliance with the HTTP/1.1 header
    /// standard, ensuring that [Key] always matches it.
    pub fn new<S: AsRef<str>>(s: S) -> Result<Self, HeaderError> {
        let s = s.as_ref();
        if !s.is_ascii() || s.is_empty() {
            Err(HeaderError)
        } else {
            Ok(Self {
                value: s.to_ascii_lowercase(),
            })
        }
    }
}
impl Display for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.value)
    }
}
impl Deref for Key {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<S: AsRef<str>> PartialEq<S> for Key {
    fn eq(&self, other: &S) -> bool {
        self.value == other.as_ref().to_ascii_lowercase()
    }
}
impl Borrow<str> for Key {
    fn borrow(&self) -> &str {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn eq_ignore_case() {
        let result = Key::new("AbC").unwrap();
        assert_eq!(result, "abc");
        assert_eq!(result, "ABC");
        assert_ne!(result, "  ABC")
    }
}
