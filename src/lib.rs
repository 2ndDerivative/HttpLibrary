use std::fmt::{Display, Formatter, Result as FmtResult};

pub mod header;
pub mod request;
pub mod response;

pub use self::{
    request::{Request, RequestMethod},
    // Traits have to be reexported due to compatibility
    response::{Code, IntoBytes, Response, ResponseType},
};

#[derive(PartialEq, Debug)]
pub struct Version(pub u64, pub u64);

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}.{}", self.0, self.1)
    }
}
