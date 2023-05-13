pub mod header;
pub mod request;
pub mod response;

pub use self::{
    request::{Request, RequestMethod},
    // Traits have to be reexported due to compatibility
    response::{Response, Byteable, ResponseCode},
};

#[derive(PartialEq, Debug)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
}
