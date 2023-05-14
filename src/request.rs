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
/// The overall HTTP request struct.
///
/// # Examples
///
/// ```
/// # use heggemann_http::{
/// #    Request,
/// #    RequestMethod,
/// #    Version,
/// #    header::Value,
/// # };
/// let input =
/// "GET /my/path HTTP/1.1\r\n\
/// Content-Length: 50\r\n\
/// Authorization: I have none\r\n
/// \r\n
/// This is somebody's body";
/// let request = input.parse::<Request>().unwrap();
///
/// assert_eq!(request.method, RequestMethod::Get);
/// assert_eq!(request.path, String::from("/my/path"));
///
/// assert_eq!(request.version, Version (1, 1));
///
/// assert_eq!(request.headers.get("content-length").unwrap(), "50");
/// assert_eq!(request.headers.get("authorization").unwrap(), "I have none");
/// ```
///
/// Header keys have to be compared in lowercase. (Work in progress)
pub struct Request {
    pub method: RequestMethod,
    pub path: String,
    pub headers: HashMap<Key, Value>,
    pub version: Version,
}

#[derive(Debug, PartialEq)]
/// Enumeration of the standardized Request methods.
///
/// Safety and Idempotency defined by the HTTP/1.1 standard.
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
    /// Safe methods are not supposed to mutate state on the server.
    /// This may be used to force a library or binary to take an
    /// immutable reference to some struct when sent a safe method.
    pub fn is_safe(&self) -> bool {
        matches!(self, Self::Get | Self::Head | Self::Options | Self::Trace)
    }
    /// An idempotent request is supposed to be non-repeatable.
    /// This includes all **safe** methods as well as `Put` and `Delete`,
    /// which are both supposed to represent only shifting between a
    /// resource existing and non existing, not incrementing or decrementing
    /// some value.
    pub fn is_idempotent(&self) -> bool {
        self.is_safe() || matches!(self, Self::Put | Self::Delete)
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

impl FromStr for Request {
    type Err = RequestParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        // Starting with a CRLF should be ignored and skipped
        // according to specification HTTP/1.1 paragraph 2.2
        let firstline = match lines.next() {
            Some("") => lines.next().ok_or(RequestParseError::EmptyRequest)?,
            None => {return Err(RequestParseError::EmptyRequest)}
            Some(x) => x
        }.split_whitespace().collect::<Vec<_>>();
        let (method_word, path, http_word) = match firstline[..3] {
            [a, b, c] => (a, b.to_string(), c),
            _ => return Err(RequestParseError::MissingStartlineElements),
        };

        let version = http_word
            .strip_prefix("HTTP/")
            .ok_or(RequestParseError::InvalidHttpWord)?
            .split_once('.')
            .and_then(|(ma, mi)| {
                Some(Version(ma.parse().ok()?, mi.parse().ok()?))
            })
            .ok_or(RequestParseError::InvalidVersion)?;

        let headers = lines.take_while(|&l| !l.is_empty()).fold(
            Ok(HashMap::new()),
            |h: Result<HashMap<Key, Value>, HeaderError>, new| {
                let Ok(mut h) = h else {
                    return h
                };
                let (key, value) = new.split_once(':')
                    .ok_or(HeaderError::NoSeparator)?;
                // This checks for pre-colon whitespace
                let key = Key::new(key)?;

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
                version: Version(1, 1),
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
                version: Version(3, 0),
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
    #[test]
    fn ignore_first_empty_line() {
        let str = "\r\nPOST /stuff HTTP/1.1\r\n\
        Some_header: A\r\n\r\n";
        let rq = str.parse::<Request>();
        assert!(rq.is_ok());
    }
    #[test]
    fn fail_empty_line() {
        let str = "";
        assert_eq!(str.parse::<Request>(), Err(RequestParseError::EmptyRequest));
    }
}
