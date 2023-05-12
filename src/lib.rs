pub mod header;
pub mod request;
pub mod response;

pub use self::{
    header::{key::Key, value::Value},
    request::{Request, RequestMethod},
    // Traits have to be reexported due to compatibility
    response::{Response, Byteable, ResponseCode},
};

#[derive(PartialEq, Debug)]
pub struct Version {
    major: u64,
    minor: u64,
}
