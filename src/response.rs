use std::collections::{hash_map::Entry, HashMap};
use std::marker::PhantomData;

use crate::header::{key::Key, value::Value, HeaderError};

#[derive(PartialEq, Debug, Clone)]
pub struct Response<S: State> {
    marker: std::marker::PhantomData<S>,
    front_matter: String,
    body: Vec<u8>,
    headers: HashMap<Key, Value>,
}

impl Response<NeedsHeaders> {
    pub fn new(code: u32) -> Self {
        Response {
            marker: PhantomData,
            front_matter: format!("HTTP/1.1 {code} {}", standard_marker_from_code(code)),
            body: vec![],
            headers: HashMap::new(),
        }
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

fn standard_marker_from_code(code: u32) -> &'static str {
    match code {
        100 => "CONTINUE",

        200 => "OK",

        400 => "BAD REQUEST",
        403 => "FORBIDDEN",
        404 => "NOT FOUND",
        405 => "METHOD NOT ALLOWED",
        411 => "LENGTH REQUIRED",
        413 => "PAYLOAD TOO LARGE",

        500 => "SERVER ERROR",
        501 => "NOT IMPLEMENTED",
        _ => todo!(),
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
        let result = Response::new(200).into_bytes();
        assert_eq!(result, b"HTTP/1.1 200 OK\r\n\r\n");
    }
    #[test]
    fn response_body_bytes() {
        let result = Response::new(200).body("SomeBODY");
        assert_eq!(result.into_bytes(), b"HTTP/1.1 200 OK\r\n\r\nSomeBODY");
    }
    #[test]
    fn response_header_bytes() {
        let result = Response::new(200).header("hi", "its me").unwrap().body("someBODY");
        assert_eq!(result.into_bytes(), b"HTTP/1.1 200 OK\r\nhi:its me\r\n\r\nsomeBODY");
    }
    #[test]
    // Header fields with different keys may appear in arbitrary order
    fn reponse_multiple_headers() {
        let result = Response::new(200)
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
        let result = Response::new(200)
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
        let r = Response::new(200);
        let result = r.clone().header(key, "no_whitespace").unwrap();
        let result2 = r.header(key, "   no_whitespace").unwrap();
        assert_eq!(result, result2);
    }
    #[test]
    fn headers_trim_trailing_whitespace() {
        let key = "some_header";
        let r = Response::new(200);
        let result = r.clone().header(key, "no_whitespace").unwrap();
        let result2 = r.header(key, "no_whitespace          ").unwrap();
        assert_eq!(result, result2);
    }
    #[test]
    fn header_cant_insert_empty() {
        assert!(Response::new(200).header("stuff", "").is_err());
        assert!(Response::new(200).header("", "stuff").is_err());
    }
}
