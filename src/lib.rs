pub mod header;
pub mod request;
pub mod response;

pub use self::{
    header::{key::Key, value::Value},
    request::{Request, RequestMethod},
    response::Response,
};

#[derive(PartialEq, Debug)]
pub struct Version {
    major: u64,
    minor: u64,
}
