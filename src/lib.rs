pub mod header;
pub mod request;
pub mod response;

pub use self::{
    header::{key::Key, value::Value, HeaderError},
    request::{Request, RequestMethod, RequestParseError},
    response::{Byteable, Response},
};

#[derive(PartialEq, Debug)]
pub struct Version {
    major: u64,
    minor: u64,
}
