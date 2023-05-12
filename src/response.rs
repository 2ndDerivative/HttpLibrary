use std::{
    collections::{hash_map::Entry, HashMap},
    error::Error,
    marker::PhantomData,
    fmt::{Display, Formatter, Result as FmtResult},
};

use crate::header::{key::Key, value::Value, HeaderError};

#[derive(PartialEq, Debug, Clone)]
pub struct Response<S: State> {
    marker: std::marker::PhantomData<S>,
    front_matter: String,
    body: Vec<u8>,
    headers: HashMap<Key, Value>,
}

impl Response<NeedsHeaders> {
    pub fn new(code: u32) -> Result<Self, InvalidCode> {
        Ok(Response {
            marker: PhantomData,
            front_matter: format!("HTTP/1.1 {code} {}", standard_phrase(code).ok_or(InvalidCode)?),
            body: vec![],
            headers: HashMap::new(),
        })
    }
    pub fn body<B: Into<Vec<u8>>>(self, body: B) -> Response<NeedsMessage> {
        Response {
            body: body.into(),
            marker: PhantomData,
            front_matter: self.front_matter,
            headers: self.headers,
        }
    }
    pub fn header<K: AsRef<str>, V: AsRef<str>>(mut self, k: K, v: V) -> Result<Self, HeaderError> {
        // Fields are required to ignore leading and trailing WS
        let (k, v) = (k.as_ref(), v.as_ref());
        match self.headers.entry(Key::new(k)?) {
            Entry::Occupied(mut x) => {
                x.get_mut().append(v)?;
            }
            Entry::Vacant(x) => {
                x.insert(Value::new(v)?);
            }
        }
        Ok(self)
    }
}

impl<S: State> Byteable for Response<S> {
    fn into_bytes(self) -> Vec<u8> {
        [
            std::iter::once(self.front_matter).chain(
                self.headers
                    .into_iter()
                    .map(|(k, v)| format!("{k}:{v}"))
            )
            .collect::<Vec<String>>()
            .join("\r\n").into_bytes(),
            "\r\n\r\n".into(),
            self.body,
        ]
        .concat()
    }
}

pub trait Byteable {
    fn into_bytes(self) -> Vec<u8>;
}

fn standard_phrase(code: u32) -> Option<&'static str> {
    match code {
        100 => Some("CONTINUE"),
        101 => Some("SWITCHING PROTOCOLS"),
        103 => Some("EARLY HINTS"),

        200 => Some("OK"),
        201 => Some("CREATED"),
        202 => Some("Accepted"),
        203 => Some("NON-AUTHORITATIVE INFORMATION"),
        204 => Some("NO CONTENT"),
        205 => Some("RESET CONTENT"),
        206 => Some("PARTIAL CONTENT"),
        207 => Some("MULTI-STATUS"),
        208 => Some("ALREADY REPORTED"),
        209 => Some("IM USED"),

        300 => Some("MULTIPLE CHOICES"),
        301 => Some("MOVED PERMANENTLY"),
        302 => Some("FOUND"),
        303 => Some("SEE OTHER"),
        304 => Some("NOT MODIFIED"),
        305 => Some("USE PROXY"),
        307 => Some("TEMPORARY REDIRECT"),
        308 => Some("PERMANENT REDIRECT"),

        400 => Some("BAD REQUEST"),
        401 => Some("UNAUTHORIZED"),
        402 => Some("PAYMENT REQUIRED"),
        403 => Some("FORBIDDEN"),
        404 => Some("NOT FOUND"),
        405 => Some("METHOD NOT ALLOWED"),
        406 => Some("NOT ACCCEPTABLE"),
        407 => Some("PROXY AUTHENTICATION REQUIRED"),
        408 => Some("REQUEST TIMEOUT"),
        409 => Some("CONFLICT"),
        410 => Some("GONE"),
        411 => Some("LENGTH REQUIRED"),
        412 => Some("PRECONDITON FAILED"),
        413 => Some("PAYLOAD TOO LARGE"),
        414 => Some("URI TOO LONG"),
        415 => Some("UNSUPPORTED MEDIA TYPE"),
        416 => Some("RANGE NOT SATISFIABLE"),
        417 => Some("EXPECTATION FAILED"),
        418 => Some("IM A TEAPOT"),
        421 => Some("MISDIRECTED REQUEST"),
        422 => Some("UNPROCESSABLE ENTITY"),
        423 => Some("LOCKED"),
        424 => Some("FAILED DEPENDENCY"),
        425 => Some("TOO EARLY"),
        426 => Some("UPGRADE REQUIRED"),
        428 => Some("PRECONDITION REQUIRED"),
        429 => Some("TOO MANY REQUESTS"),
        431 => Some("REQUEST HEADER FIELDS TOO LARGE"),
        451 => Some("UNAVAILABLE FOR LEGAL REASONS"),

        500 => Some("SERVER ERROR"),
        501 => Some("NOT IMPLEMENTED"),
        502 => Some("BAD GATEWAY"),
        503 => Some("SERVICE UNAVAILABLE"),
        504 => Some("GATEWAY TIMEOUT"),
        505 => Some("HTTP VERSION NOT SUPPORTED"),
        506 => Some("VARIANT ALSO NEGOTIATES"),
        507 => Some("INSUFFICIENT STORAGE"),
        508 => Some("LOOP DETECTED"),
        510 => Some("NOT EXTENDED"),
        511 => Some("NETWORK AUTHENTICATION REQUIRED"),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InvalidCode;
impl Error for InvalidCode {}
impl Display for InvalidCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "invalid response code")
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum NeedsHeaders {}
impl State for NeedsHeaders {}
#[derive(Debug, PartialEq, Clone)]
pub enum NeedsMessage {}
impl State for NeedsMessage {}
pub trait State {}

#[cfg(test)]
mod tests {
    use crate::response::Response;

    use super::*;

    #[test]
    fn response_title_bytes() {
        let result = Response::new(200).unwrap().into_bytes();
        assert_eq!(result, b"HTTP/1.1 200 OK\r\n\r\n");
    }
    #[test]
    fn response_body_bytes() {
        let result = Response::new(200).unwrap().body("SomeBODY");
        assert_eq!(result.into_bytes(), b"HTTP/1.1 200 OK\r\n\r\nSomeBODY");
    }
    #[test]
    fn response_header_bytes() {
        let result = Response::new(200).unwrap().header("hi", "its me").unwrap().body("someBODY");
        assert_eq!(result.into_bytes(), b"HTTP/1.1 200 OK\r\nhi:its me\r\n\r\nsomeBODY");
    }
    #[test]
    // Header fields with different keys may appear in arbitrary order
    fn reponse_multiple_headers() {
        let result = Response::new(200).unwrap()
            .header("hey", "man").unwrap()
            .header("how", "are you").unwrap()
            .body("someBODY");
        assert!(result.clone().into_bytes()
            == b"HTTP/1.1 200 OK\r\nhey:man\r\nhow:are you\r\n\r\nsomeBODY"
            || result.into_bytes()
            == b"HTTP/1.1 200 OK\r\nhow:are you\r\nhey:man\r\n\r\nsomeBODY"
        )
    }
    #[test]
    fn multiple_headers() -> Result<(), HeaderError> {
        let result = Response::new(200).unwrap()
            .header("stuff", "Aaron")?
            .header("STUFF", "Berta")?
            .header("sTuFf", "Charlie   ")?
            .header("other_stuff", "Daniel")?;
        assert_eq!(
            result.headers.get("stuff"),
            Some(&Value::new("Aaron,Berta,Charlie").unwrap())
        );
        assert_eq!(
            result.headers.get("other_stuff"),
            Some(&Value::new("Daniel").unwrap())
        );
        Ok(())
    }
    #[test]
    fn headers_trim_leading_whitespace() {
        let key = "some_header";
        let r = Response::new(200).unwrap();
        let result = r.clone().header(key, "no_whitespace").unwrap();
        let result2 = r.header(key, "   no_whitespace").unwrap();
        assert_eq!(result, result2);
    }
    #[test]
    fn headers_trim_trailing_whitespace() {
        let key = "some_header";
        let r = Response::new(200).unwrap();
        let result = r.clone().header(key, "no_whitespace").unwrap();
        let result2 = r.header(key, "no_whitespace          ").unwrap();
        assert_eq!(result, result2);
    }
    #[test]
    fn header_cant_insert_empty() {
        assert!(Response::new(200).unwrap().header("stuff", "").is_err());
        assert!(Response::new(200).unwrap().header("", "stuff").is_err());
    }
}
