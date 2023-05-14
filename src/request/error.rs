use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FMTResult},
};

use crate::{header::{HeaderError, KeyError}, Response};

#[derive(Debug, PartialEq)]
/// Collects all Errors that may happen during request parsing.
/// 
/// Can include both incompliance with the standard and failed parsing from the servers side.
/// 
/// If standard-compliant return codes are intended, the `appropriate_reponse()` method on this
/// type can return a response that the standard proscribes or strongly recommends for the kind of
/// semantic or syntactic error that was found.
/// 
/// For the case that there is no standardized response, `appropriate_response()` returns an `Option`
/// that should be handled.
pub enum RequestParseError {
    /// The request is an empty or whitespace-only string
    EmptyRequest,
    /// The request is missing any of `method request-target HTTP-version`
    MissingStartlineElements,
    /// The third word in the start line does not start with "HTTP/"
    InvalidHttpWord,
    /// The method has not been recognized. A server having this error should
    /// return a [501][crate::Response::NotImplemented]
    MethodNotRecognized(MethodParseError),
    /// A header is not compliant with header syntax
    BadHeader(HeaderError),
    /// The version word in the (`HTTP/[major].[minor]`)-term is
    /// not parseable as such
    InvalidVersion,
}
impl RequestParseError {
    #[must_use]
    pub fn appropriate_response(&self) -> Option<Response> {
        match self {
            Self::MethodNotRecognized(_) => Some(Response::NotImplemented),
            Self::BadHeader(HeaderError::Key(KeyError::ColonWhitespace)) => Some(Response::BadRequest),
            _ => None
        }
    }
}
impl Error for RequestParseError {}
impl Display for RequestParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FMTResult {
        write!(
            f,
            "{}",
            match self {
                Self::EmptyRequest => "empty string".to_owned(),
                Self::MissingStartlineElements => "request is missing any of method request-target HTTP-version".to_owned(),
                Self::InvalidHttpWord => "start line does not end with a HTTP/.. version string".to_owned(),
                Self::MethodNotRecognized(e) => format!("method not recognized: {}", e),
                Self::BadHeader(_) => "header invalid".to_owned(),
                Self::InvalidVersion => "version invalid".to_owned(),
            }
        )
    }
}
impl From<MethodParseError> for RequestParseError {
    fn from(value: MethodParseError) -> Self {
        RequestParseError::MethodNotRecognized(value)
    }
}
impl From<HeaderError> for RequestParseError {
    fn from(value: HeaderError) -> Self {
        RequestParseError::BadHeader(value)
    }
}

#[derive(Debug, PartialEq)]
/// Ascii-uppercase is not technically a must for new HTTP methods,
/// but all the standardized methods are by said standard all
/// uppercased.
pub enum MethodParseError {
    NotAsciiUppercase,
    NotAMethod,
}
impl Error for MethodParseError {}
impl Display for MethodParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FMTResult {
        write!(
            f,
            "{}",
            match self {
                Self::NotAsciiUppercase => "not ascii uppercase",
                Self::NotAMethod => "not a method word",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appropriate_reponse_method_not_recognized() {
        assert_eq!(RequestParseError::MethodNotRecognized(
            MethodParseError::NotAMethod
        ).appropriate_response(), Some(Response::NotImplemented))
    }
}