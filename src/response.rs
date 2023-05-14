use std::{
    collections::{hash_map::Entry, HashMap},
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    marker::PhantomData,
    string::FromUtf8Error,
};

use crate::{
    header::{key::Key, value::Value, HeaderError},
    Version,
};

pub trait ResponseCode {
    fn response_type(&self) -> Response;
    fn code(&self) -> u16 {
        self.response_type() as u16
    }
    fn standard_phrase(&self) -> &'static str {
        standard_phrase(self.response_type() as u16).unwrap()
    }
}

pub trait IntoBytes {
    fn into_bytes(self) -> Vec<u8>;
    fn max_version(&self) -> Version;
}

impl<T: IntoBytes + ResponseCode> FirstLine for T {}

trait FirstLine: IntoBytes + ResponseCode {
    fn first_line(&self) -> String {
        format!(
            "HTTP/{}.{} {} {}",
            self.max_version().0,
            self.max_version().1,
            self.code(),
            self.standard_phrase()
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
/// Standard HTTP Response struct
/// write in raw bytes using `into_bytes()`, as the HTTP standard does not
/// require valid UTF response bodies.
///
/// # Examples
/// ```
/// # use heggemann_http::{
/// #     Response,
/// #     IntoBytes
/// # };
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let res = dbg!(Response::Ok
///     .header("Host", "github.com:80")?
///     .body("this is some body"));
/// dbg!(res.max_version());
/// assert_eq!(res.to_string(),
///     "HTTP/1.1 200 OK\r\n\
///     host:github.com:80\r\n\r\n\
///     this is some body");
/// # Ok(())
/// # }
/// ```
pub enum Response {
    /// ## 100 CONTINUE
    /// The server has received the request headers and the client should proceed to send
    /// the request body (if a body needs to be sent).
    ///
    /// Usually returned after receiving an `expect: 100-continue` header from the client.
    Continue = 100,
    /// ## 101 SWITCHING PROTOCOLS
    /// The server agrees to switching protocols after a request asking for it.
    SwitchingProtocols = 101,
    /// ## 102 PROCESSING
    /// WebDAV request may take a long time. This indicates a long time before the answer is to be
    /// expected by the client.
    #[deprecated]
    Processing = 102,
    /// ## 103 EARLY HINTS
    /// For returning response headers before the final HTTP message.
    EarlyHints = 103,

    /// ## 200 OK
    /// Standard reponse for successful requests. The actual reponse may depend on the request method.
    /// Contains the asked-for data in a [GET][crate::request::RequestMethod::Get] request.
    /// Contains an overview of the change made in a [POST][crate::request::RequestMethod::Post] request.
    Ok = 200,
    /// ## 201 CREATED
    /// A new resource has been created, for example after a [PUT][crate::request::RequestMethod::Put] request.
    Created = 201,
    /// ## 202 ACCEPTED
    /// The request has been accepted, but is still processing. May fail later on.
    Accepted = 202,
    /// ## 203 NON-AUTHORITATIVE INFORMATION
    /// The server is returning the answer from a proxy, but in a modified manner.
    NonAuthoritativeInformation = 203,
    /// ## 204 NO CONTENT
    /// Successful on a method/resource with no content.
    NoContent = 204,
    /// ## 205 RESET CONTENT
    /// The server processed the request and asks the user for resetting their view. No content
    /// is returned.
    ResetContent = 205,
    /// ## 206 PARTIAL CONTENT
    /// The server is delivering a part of the resource **because of a client range header**.
    /// The range header is used to resume interrupted downloads or split a download into multiple
    /// parallel streams.
    PartialContent = 206,
    /// ## 207 MULTI-STATUS
    /// The message body is an `XML`-message by default and can contain a multitude of response
    /// codes depending on the subrequests made.
    MultiStatus = 207,
    /// ## 208 ALREADY REPORTED
    /// The members of a DAV binding have already been enumerated in a preceding part of the
    /// (multistatus) response, and are not being repeated.
    AlreadyReported = 208,
    /// ## 226 IM USED
    /// The server has fulfilled a request for the resource, and the response is a representation
    /// of the result of one or more instance-manipulations applied to the current instance.
    ImUsed = 226,

    /// ## 300 MULTIPLE CHOICES
    /// Indicates that multiple choices for the resource are available.
    /// One example are different video formats or file extensions.
    MultipleChoices = 300,
    /// ## 301 MOVED PERMANENTLY
    /// This request and all future requests should be directed to the new URI.
    MovedPermanently = 301,
    /// ## 302 FOUND
    /// ### Previously: "Moved Temporarily"
    /// The server tells the client to move to another URL. In HTTP/1.0, the client was supposed
    /// to do it with the same method, but most browsers change to get when receiving a `302`.
    /// To change this behaviour, [303][Response::SeeOther] and [307][Response::TemporaryRedirect]
    /// may be used.
    Found = 302,
    /// ## 303 SEE OTHER
    /// The response to this request can be found under another URI using [GET][crate::request::RequestMethod::Get]. When received
    /// in response to [POST][crate::request::RequestMethod::Post], [PUT][crate::request::RequestMethod::Put] or [DELETE][crate::request::RequestMethod::Delete], the client should presume that the server has
    /// received the data and should issue a new [GET][crate::request::RequestMethod::Get] request to the given URI.
    SeeOther = 303,
    /// ## 304 NOT MODIFIED
    /// Indicates that the resource has not been modified since the version specified by
    /// the request headers `If-Modified-Since` or `If-None-Match`. In such case, there
    /// is no need to retransmit the resource since the client still has a previously-
    /// downloaded copy.
    NotModified = 304,
    /// ## 305 USE PROXY
    /// The requested resource is available only through a proxy, the address for which is
    /// provided in the response. For security reasons, many HTTP clients do not obey this
    /// status code.
    UseProxy = 305,
    /// ## 306 SWITCH PROXY
    /// No longer used. Meant "subsequent requests should use the specified proxy".
    #[deprecated]
    SwitchProxy = 306,
    /// ## 307 TEMPORARY REDIRECT
    /// The request should be repeated with another URI, but future request should use this URI.
    /// In contrast to how [302][Response::Found] was historically implemented, the request
    /// method is **not allowed** to be changed when reissuing the original request. For example,
    /// a [POST][crate::request::RequestMethod::Post] request should be repeated using another [POST][crate::request::RequestMethod::Post] request.
    TemporaryRedirect = 307,
    /// ## 308 PERMANENT REDIRECT
    /// This and all future requests should be directed to the given URI. 308 parallels the
    /// behaviour of [301][Response::MovedPermanently], but *does not allow the HTTP method
    /// to change*. So, for example, submitting a form to a permanently redirected resource
    /// may continue smoothly.
    PermanentRedirect = 308,

    /// ## 400 BAD REQUEST
    /// The server cannot or will not process the request due to an apparent client error
    /// (e.g. malformed request syntax, size too large, invalid request message framing,
    /// or deceptive request routing).
    BadRequest = 400,
    /// ## 401 UNAUTHORIZED
    /// Similar to [403][Response::Forbidden], but specifically for use when authentication
    /// is required and as failed or has not yet been provided. The response must include a
    /// WWW-Authenticate header field containing a challenge applicatble to the requested
    /// resource. 401 semantically means *unauthorised*, the user does not have valid
    /// authentication credentials for the target resource. Some sites incorrectly issue 401
    /// when an IP address is banned from the website and that specific address is refused
    /// permission to access a website.
    Unauthorized = 401,
    /// ## 402 PAYMENT REQUIRED
    /// Reserved for future use. The original intention was that this code might be used as
    /// part of some form of digital cash or micropayment scheme, as proposed, for example,
    /// by GNU Taler, but this has not yet happened
    PaymentRequired = 402,
    /// ## 403 FORBIDDEN
    /// The request contained valid data and was understood by the server, but it is refusing
    /// action. This may be due to insufficient user permissions or to needing an account.
    /// Also is used for prohibited action (duplicate records etc.).
    Forbidden = 403,
    /// ## 404 NOT FOUND
    /// The requested resource could is not available, but may be in the future. Subsequent
    /// requests are permissible.
    NotFound = 404,
    /// ## 405 METHOD NOT ALLOWED
    /// A request method is not supported for the requested resource, for example, a [GET][crate::request::RequestMethod::Get] request
    /// on a form that requires a [POST][crate::request::RequestMethod::Post], or a [PUT][crate::request::RequestMethod::Put] on a read-only resource.
    MethodNotAllowed = 405,
    /// ## 406 NOT ACCEPTABLE
    /// The requested resource is capable of generating only content not acceptable according
    /// to the Accept headers sent by the client.
    NotAcceptable = 406,
    /// ## 407 PROXY AUTHENTICATION REQUIRED
    /// The client must first authenticate itself with the proxy.
    ProxyAuthenticationRequired = 407,
    /// ## 408 REQUEST TIMEOUT
    /// The server timed out waiting for the request. The client did not produce a request in
    /// the time the server took to wait. The client may repeat the request later.
    RequestTimeout = 408,
    /// ## 409 CONFLICT
    /// The request could not be processed because of conflict in the current resource state.
    /// This may be an edit- or concurrency conflict.
    Conflict = 409,
    /// ## 410 GONE
    /// The resource is permanently gone. The client should not reattempt this request.
    /// Search features and engines should remove this entry. If purging this record is not
    /// wished, [404][Response::NotFound] should be used.
    Gone = 410,
    /// ## 411 LENGTH REQUIRED
    /// The request did not specify the length of its content, which is required by the requested
    /// resource.
    LengthRequired = 411,
    /// ## 412 PRECONDITON FAILED
    /// The server does not meet one of the preconditions that the requester put on the request
    /// header fields.
    PreconditonFailed = 412,
    /// ## 413 PAYLOAD TOO LARGE
    /// ### Previously: "Request Entity Too Large"
    /// The request is larger than the server is willing or able to process.
    PayloadTooLarge = 413,
    /// ## 414 URI TOO LONG
    /// The URI was too long for the server to process. May be due to long [GET][crate::request::RequestMethod::Get] query strings.
    UriTooLong = 414,
    /// ## 415 UNSUPPORTED MEDIATYPE
    /// The media type by the request entity is not supported by the server, such as not
    /// accepting a certain image format.
    UnsupportedMediaType = 415,
    /// ## 416 RANGE NOT SATISFIABLE
    /// The client asked for a portion of the resource that the server could not supply.
    /// This is a client error, so it has to be unreachable or otherwise not satisfiable.
    RangeNotSatisfiable = 416,
    /// ## 417 EXPECTATION FAILED
    /// The server cannot meet the requirements of the Expect request-header field.
    ExpectationFailed = 417,
    /// ## 418 IM A TEAPOT
    /// A joke response code.
    ImATeapot = 418,
    /// ## 421 MISDIRECTED REQUEST
    /// The request was directed at a server that is not able to produce a response
    /// (for example because of connection reuse).
    MisdirectedRequest = 421,
    /// ## 422 UNPROCESSABLE ENTITY
    /// The request was well-formed but was unable to be followed due to semantic errors.
    UnprocessableEntity = 422,
    /// ## 423 LOCKED
    /// The resource that is being accessed is locked.
    Locked = 423,
    /// ## 424 FAILED DEPENDENCY
    /// The request failed because it depended on another request and that request failed.
    FailedDependency = 424,
    /// ## 425 TOO EARLY
    /// Indicates that the server is unwilling to risk processing a request that might
    /// be replayed.
    TooEarly = 425,
    /// ## 426 UPGRADE REQUIRED
    /// The client should switch to a different protocol given in the `Upgrade` header field.
    UpgradeRequired = 426,
    /// ## 428 PRECONDITION REQUIRED
    /// The origin server requires the request to be conditional. Intended to prevent the
    /// 'lost update' problem, where a client [GET][crate::request::RequestMethod::Get]s a resource's state, modifies it, and
    /// [PUT][crate::request::RequestMethod::Put]s it back to the server, when meanwhile a third party has modified the state
    /// on the server, leading to a conflict.
    PreconditionRequired = 428,
    /// ## 429 TOO MANY REQUESTS
    /// The user has sent too many requests in a given amount of time. Intended for use
    /// with rate-limiting schemes.
    TooManyRequests = 429,
    /// ## 431 REQUEST HEADER FIELDS TOO LARGE
    /// The server is unwilling to process the request because either an individual
    /// header field, or all the header fields collectively, are too large.
    RequestHeaderFieldsTooLarge = 431,
    /// ## 451 UNAVAILABLE FOR LEGAL REASONS
    /// A server operator has received a legal demand to deny access to a resource
    /// or to a set of resources that includes the requested resource.
    UnavailableForLegalReasons = 451,

    /// ## 500 SERVER ERROR
    /// A generic error message, given when an unexpected condition was encountered
    /// and no more specific message is suitable.
    ServerError = 500,
    /// ## 501 NOT IMPLEMENTED
    /// The server either does not recognize the request method, or it lacks the
    /// ability to fulfil the request.
    NotImplemented = 501,
    /// ## 502 BAD GATEWAY
    /// The server was acting as a gateway or proxy and received an invalid response
    /// from the upstream server.
    BadGateway = 502,
    /// ## 503 SERVICE UNAVAILABLE
    /// The server cannot handle the request (because it is overloaded or down for
    /// maintenance).
    ServiceUnavailable = 503,
    /// ## 504 GATEWAY TIMEOUT
    /// The server was acting as a gateway or proxy and did not receive a timely
    /// response from the upstream server.
    GatewayTimeout = 504,
    /// ## 505 HTTP VERSION NOT SUPPORTED
    /// The server does not support the HTTP version used in the request.
    HttpVersionNotSupported = 505,
    /// ## 506 VARIANT ALSO NEGOTIATES
    /// Transparent content negotiation for the request results in a circular reference.
    VariantAlsoNegotiates = 506,
    /// ## 507 INSUFFICIENT STORAGE
    /// The server is unable to store the representation needed to complete the request.
    InsufficientStorage = 507,
    /// ## 508 LOOP DETECTED
    /// The server detected an infinite loop while processing the request (sent instead
    /// of [208 Already Reported][Response::AlreadyReported]).
    LoopDetected = 508,
    /// ## 510 NOT EXTENDED
    /// Further extensions to the request are required for the server to fulfil it.
    NotExtended = 510,
    /// ## 511 NETWORK AUTHENTICATION REQUIRED
    /// The client needs to authenticate to gain network access. Intended for use by
    /// intercepting proxies used to control access to the network
    NetworkAuthenticationRequired = 511,
}

impl Response {
    pub fn new(code: u16) -> Result<Self, InvalidCode> {
        Response::try_from(code)
    }
    pub fn body<B: Into<Vec<u8>>>(self, body: B) -> ResponseBuilder<Complete> {
        ResponseBuilder {
            response: self,
            marker: PhantomData,
            body: body.into(),
            headers: HashMap::new(),
        }
    }
    pub fn header<K: AsRef<str>, V: AsRef<str>>(
        self,
        k: K,
        v: V,
    ) -> Result<ResponseBuilder<Incomplete>, HeaderError> {
        let (k, v) = (k.as_ref(), v.as_ref());
        let headers = HashMap::from([(Key::new(k)?, Value::new(v)?)]);
        Ok(ResponseBuilder {
            response: self,
            marker: PhantomData,
            body: vec![],
            headers,
        })
    }
}

impl ResponseCode for Response {
    fn response_type(&self) -> Response {
        self.clone()
    }
}

impl IntoBytes for Response {
    fn into_bytes(self) -> Vec<u8> {
        String::from(self).into()
    }
    fn max_version(&self) -> Version {
        Version(1, 0)
    }
}

impl From<Response> for String {
    fn from(value: Response) -> Self {
        format!("{value}")
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}\r\n\r\n", self.first_line())
    }
}

impl From<Response> for Vec<u8> {
    fn from(value: Response) -> Self {
        value.into_bytes()
    }
}

impl TryFrom<u16> for Response {
    type Error = InvalidCode;
    #[allow(deprecated)]
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            100 => Ok(Self::Continue),
            101 => Ok(Self::SwitchingProtocols),
            102 => Ok(Self::Processing),
            103 => Ok(Self::EarlyHints),

            200 => Ok(Self::Ok),
            201 => Ok(Self::Created),
            202 => Ok(Self::Accepted),
            203 => Ok(Self::NonAuthoritativeInformation),
            204 => Ok(Self::NoContent),
            205 => Ok(Self::ResetContent),
            206 => Ok(Self::PartialContent),
            207 => Ok(Self::MultiStatus),
            208 => Ok(Self::AlreadyReported),
            226 => Ok(Self::ImUsed),

            300 => Ok(Self::MultipleChoices),
            301 => Ok(Self::MovedPermanently),
            302 => Ok(Self::Found),
            303 => Ok(Self::SeeOther),
            304 => Ok(Self::NotModified),
            305 => Ok(Self::UseProxy),
            306 => Ok(Self::SwitchProxy),
            307 => Ok(Self::TemporaryRedirect),
            308 => Ok(Self::PermanentRedirect),

            400 => Ok(Self::BadRequest),
            401 => Ok(Self::Unauthorized),
            402 => Ok(Self::PaymentRequired),
            403 => Ok(Self::Forbidden),
            404 => Ok(Self::NotFound),
            405 => Ok(Self::MethodNotAllowed),
            406 => Ok(Self::NotAcceptable),
            407 => Ok(Self::ProxyAuthenticationRequired),
            408 => Ok(Self::RequestTimeout),
            409 => Ok(Self::Conflict),
            410 => Ok(Self::Gone),
            411 => Ok(Self::LengthRequired),
            412 => Ok(Self::PreconditonFailed),
            413 => Ok(Self::PayloadTooLarge),
            414 => Ok(Self::UriTooLong),
            415 => Ok(Self::UnsupportedMediaType),
            416 => Ok(Self::RangeNotSatisfiable),
            417 => Ok(Self::ExpectationFailed),
            418 => Ok(Self::ImATeapot),
            421 => Ok(Self::MisdirectedRequest),
            422 => Ok(Self::UnprocessableEntity),
            423 => Ok(Self::Locked),
            424 => Ok(Self::FailedDependency),
            425 => Ok(Self::TooEarly),
            426 => Ok(Self::UpgradeRequired),
            428 => Ok(Self::PreconditionRequired),
            429 => Ok(Self::TooManyRequests),
            431 => Ok(Self::RequestHeaderFieldsTooLarge),
            451 => Ok(Self::UnavailableForLegalReasons),

            500 => Ok(Self::ServerError),
            501 => Ok(Self::NotImplemented),
            502 => Ok(Self::BadGateway),
            503 => Ok(Self::ServiceUnavailable),
            504 => Ok(Self::GatewayTimeout),
            505 => Ok(Self::HttpVersionNotSupported),
            506 => Ok(Self::VariantAlsoNegotiates),
            507 => Ok(Self::InsufficientStorage),
            508 => Ok(Self::LoopDetected),
            510 => Ok(Self::NotExtended),
            511 => Ok(Self::NetworkAuthenticationRequired),
            _ => Err(InvalidCode),
        }
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

#[derive(PartialEq, Debug, Clone)]
pub struct ResponseBuilder<S: State> {
    response: Response,
    marker: std::marker::PhantomData<S>,
    body: Vec<u8>,
    headers: HashMap<Key, Value>,
}

impl<S: State> ResponseCode for ResponseBuilder<S> {
    fn response_type(&self) -> Response {
        self.response.clone()
    }
}

impl ResponseBuilder<Incomplete> {
    pub fn body<B: Into<Vec<u8>>>(self, body: B) -> ResponseBuilder<Complete> {
        let body = body.into();
        ResponseBuilder {
            response: self.response,
            marker: PhantomData,
            body,
            headers: self.headers,
        }
    }
    pub fn header<K: AsRef<str>, V: AsRef<str>>(
        mut self,
        k: K,
        v: V,
    ) -> Result<ResponseBuilder<Incomplete>, HeaderError> {
        let k = Key::new(k.as_ref())?;
        match self.headers.entry(k) {
            Entry::Occupied(mut e) => {
                e.get_mut().append(v.as_ref())?;
            }
            Entry::Vacant(e) => {
                e.insert(Value::new(v.as_ref())?);
            }
        }
        Ok(self)
    }
}

impl<S: State> IntoBytes for ResponseBuilder<S> {
    fn into_bytes(self) -> Vec<u8> {
        [
            std::iter::once(self.first_line())
                .chain(self.headers.into_iter().map(|(k, v)| format!("{k}:{v}")))
                .collect::<Vec<String>>()
                .join("\r\n")
                .into_bytes(),
            "\r\n\r\n".into(),
            self.body,
        ]
        .concat()
    }
    fn max_version(&self) -> Version {
        let k = Key::new("host").unwrap();
        if self.headers.contains_key(&k) {
            Version(1, 1)
        } else {
            Version(1, 0)
        }
    }
}

impl<S: State> TryFrom<ResponseBuilder<S>> for String {
    type Error = FromUtf8Error;
    fn try_from(value: ResponseBuilder<S>) -> Result<Self, Self::Error> {
        String::from_utf8(value.into_bytes())
    }
}

impl<S: State> From<ResponseBuilder<S>> for Vec<u8> {
    fn from(value: ResponseBuilder<S>) -> Self {
        value.into_bytes()
    }
}

impl<S: State> Display for ResponseBuilder<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}\r\n\r\n{}",
            std::iter::once(self.first_line())
                .chain(self.headers.iter().map(|(k, v)| format!("{k}:{v}")))
                .collect::<Vec<_>>()
                .join("\r\n"),
            String::from_utf8(self.body.clone()).unwrap_or_else(|_| { format!("{:?}", self.body) })
        )
    }
}

impl From<ResponseBuilder<Incomplete>> for ResponseBuilder<Complete> {
    fn from(value: ResponseBuilder<Incomplete>) -> Self {
        value.body("")
    }
}

pub fn standard_phrase(code: u16) -> Option<&'static str> {
    match code {
        100 => Some("CONTINUE"),
        101 => Some("SWITCHING PROTOCOLS"),
        102 => Some("PROCESSING"),
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
        226 => Some("IM USED"),

        300 => Some("MULTIPLE CHOICES"),
        301 => Some("MOVED PERMANENTLY"),
        302 => Some("FOUND"),
        303 => Some("SEE OTHER"),
        304 => Some("NOT MODIFIED"),
        305 => Some("USE PROXY"),
        306 => Some("SWITCH PROXY"),
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

#[derive(Debug, PartialEq, Clone)]
pub enum Incomplete {}
impl State for Incomplete {}
#[derive(Debug, PartialEq, Clone)]
pub enum Complete {}
impl State for Complete {}
pub trait State {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_title_bytes() {
        let result = Response::Ok.into_bytes();
        assert_eq!(result, b"HTTP/1.0 200 OK\r\n\r\n");
    }
    #[test]
    fn response_body_bytes() {
        let result = Response::Ok.body("SomeBODY");
        assert_eq!(result.into_bytes(), b"HTTP/1.0 200 OK\r\n\r\nSomeBODY");
    }
    #[test]
    fn response_header_bytes() {
        let result = Response::Ok
            .header("hi", "its me")
            .unwrap()
            .body("someBODY");
        assert_eq!(
            result.into_bytes(),
            b"HTTP/1.0 200 OK\r\nhi:its me\r\n\r\nsomeBODY"
        );
    }
    #[test]
    // Header fields with different keys may appear in arbitrary order
    fn reponse_multiple_headers() {
        let result = Response::Ok
            .header("hey", "man")
            .unwrap()
            .header("how", "are you")
            .unwrap()
            .body("someBODY");
        assert!(
            result.clone().into_bytes()
                == b"HTTP/1.0 200 OK\r\nhey:man\r\nhow:are you\r\n\r\nsomeBODY"
                || result.into_bytes()
                    == b"HTTP/1.0 200 OK\r\nhow:are you\r\nhey:man\r\n\r\nsomeBODY"
        )
    }
    #[test]
    fn multiple_headers() -> Result<(), HeaderError> {
        let result = Response::Ok
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
        let r = Response::Ok;
        let result = r.clone().header(key, "no_whitespace").unwrap();
        let result2 = r.header(key, "   no_whitespace").unwrap();
        assert_eq!(result, result2);
    }
    #[test]
    fn headers_trim_trailing_whitespace() {
        let key = "some_header";
        let r = Response::Ok;
        let result = r.clone().header(key, "no_whitespace").unwrap();
        let result2 = r.header(key, "no_whitespace          ").unwrap();
        assert_eq!(result, result2);
    }
    #[test]
    fn header_cant_insert_empty() {
        assert!(Response::Ok.header("stuff", "").is_err());
        assert!(Response::Ok.header("", "stuff").is_err());
    }
    #[test]
    fn try_into_string() -> Result<(), Box<dyn std::error::Error>> {
        let response = Response::new(404)?;
        let response = response.header("your", "mom")?.body("is great");
        let string: String = response.try_into()?;
        assert_eq!(
            string,
            "HTTP/1.0 404 NOT FOUND\r\n\
            your:mom\r\n\r\n\
            is great"
                .to_owned()
        );
        Ok(())
    }
    #[test]
    fn complete_correct_string() {
        let test_string = "HTTP/1.0 400 BAD REQUEST\r\n\
        header:stuff\r\n\r\n"
            .to_owned();
        let raw = Response::BadRequest.header("header", "stuff").unwrap();
        assert_eq!(raw.to_string(), test_string);
        assert_eq!(raw.body("").to_string(), test_string)
    }
    #[test]
    fn print_invalid_utf8() {
        let test_string = "HTTP/1.0 400 BAD REQUEST\r\n\r\n\
        [14, 147, 94]"
            .to_owned();
        let response = Response::BadRequest.body(vec![14, 147, 94]);
        assert_eq!(test_string, response.to_string());
    }
    #[test]
    fn print_no_header_only_two_rns() {
        let test_string = "HTTP/1.0 418 IM A TEAPOT\r\n\r\n".to_owned();
        let response = Response::ImATeapot;
        assert_eq!(test_string, response.to_string())
    }
    #[test]
    fn version_host_key() {
        let res = Response::Ok.header("Host", "github.com").unwrap();
        assert_eq!(res.max_version(), Version(1, 1));
    }
    #[test]
    fn version_no_host_key() {
        let res = Response::Ok;
        assert_eq!(res.max_version(), Version(1, 0));
    }
}
