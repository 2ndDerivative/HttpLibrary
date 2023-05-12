use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

pub mod key;
pub mod value;

#[derive(PartialEq, Debug)]
pub struct HeaderError;
impl Error for HeaderError {}
impl Display for HeaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "invalid header format")
    }
}
