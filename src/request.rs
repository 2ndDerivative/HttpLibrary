use std::{
    collections::{hash_map::Entry, HashMap},
    error::Error,
    fmt::{Display, Formatter, Result as FMTResult},
    str::FromStr,
};

use crate::{
    header::{key::Key, value::Value, HeaderError},
    Version,
};

#[derive(Debug, PartialEq)]
pub struct Request {
    pub method: RequestMethod,
    pub path: String,
    pub headers: HashMap<Key, Value>,
    pub version: Version,
}

#[derive(Debug, PartialEq)]
pub enum RequestMethod {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
}

impl RequestMethod {
    pub fn is_safe(&self) -> bool {
        matches!(self, Self::Get | Self::Head | Self::Options | Self::Trace)
    }
    pub fn is_idempotent(&self) -> bool {
        self.is_safe() || matches!(self, Self::Put | Self::Delete)
    }
}

#[derive(Debug, PartialEq)]
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

impl FromStr for RequestMethod {
    type Err = MethodParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !(s.chars().all(|c| c.is_ascii_uppercase())) {
            return Err(MethodParseError::NotAsciiUppercase);
        };
        match s {
            "GET" => Ok(Self::Get),
            "HEAD" => Ok(Self::Head),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "CONNECT" => Ok(Self::Connect),
            "OPTIONS" => Ok(Self::Options),
            "TRACE" => Ok(Self::Trace),
            _ => Err(MethodParseError::NotAMethod),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum RequestParseError {
    EmptyRequest,
    NoMethod,
    NoPath,
    NoHttpWord,
    MethodNotRecognized(MethodParseError),
    BadHeader(HeaderError),
    InvalidVersion,
}
impl Error for RequestParseError {}
impl Display for RequestParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FMTResult {
        write!(
            f,
            "{}",
            match self {
                Self::EmptyRequest => "empty string".to_owned(),
                Self::NoMethod => "no method".to_owned(),
                Self::NoPath => "no path".to_owned(),
                Self::NoHttpWord => "no version".to_owned(),
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

impl FromStr for Request {
    type Err = RequestParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let mut firstline = lines
            .next()
            .ok_or(RequestParseError::EmptyRequest)?
            .split_whitespace();
        let method_word = firstline.next().ok_or(RequestParseError::NoMethod)?;
        let path = firstline
            .next()
            .ok_or(RequestParseError::NoPath)?
            .to_string();
        let http_word = firstline.next().ok_or(RequestParseError::NoHttpWord)?;
        let version = match http_word
            .strip_prefix("HTTP/")
            .map(|x| x.split('.').map(|x| x.parse::<u64>()).collect::<Vec<_>>())
            .as_deref()
        {
            Some([Ok(major), Ok(minor)]) => Version {
                major: *major,
                minor: *minor,
            },
            _ => return Err(RequestParseError::InvalidVersion),
        };
        let headers = lines.take_while(|&l| !l.is_empty()).fold(
            Ok(HashMap::new()),
            |h: Result<HashMap<Key, Value>, HeaderError>, new| {
                let Ok(mut h) = h else {
                    return h
                };
                let mut parts = new.split(':');
                let key = Key::new(parts.next().ok_or(HeaderError)?)?;
                let value = parts.next().ok_or(HeaderError)?;

                match h.entry(key) {
                    Entry::Occupied(mut x) => x.get_mut().append(value)?,
                    Entry::Vacant(x) => {
                        x.insert(Value::new(value)?);
                    }
                };
                Ok(h)
            },
        )?;
        let method = method_word.parse()?;
        Ok(Request {
            method,
            path,
            headers,
            version,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_one_one() {
        let request = "GET / HTTP/1.1\r\n".parse().unwrap();
        assert!(matches!(
            request,
            Request {
                method: RequestMethod::Get,
                version: Version { major: 1, minor: 1 },
                ..
            }
        ))
    }
    #[test]
    fn version_three() {
        let request = "POST /stuff HTTP/3.0\r\n\r\n".parse().unwrap();
        assert!(matches!(
            request,
            Request {
                version: Version { major: 3, minor: 0 },
                ..
            }
        ))
    }
    #[test]
    fn version_invalid_three_items() {
        let request = "DELETE /other/stuff HTTP/2.0.1\r\n".parse::<Request>();
        assert_eq!(request, Err(RequestParseError::InvalidVersion))
    }
    #[test]
    fn headers_combine() {
        let request = "POST /stuff HTTP/1.1\r\n\
            Some_header: A\r\n\
            Some_Header: B\r\n\
            some_header:    C    \r\n"
            .parse::<Request>()
            .unwrap();
        assert_eq!(request.headers.get("some_header").unwrap(), "A,B,C");
    }
}
