use std::fmt::Formatter;

#[derive(Debug)]
pub enum Error {
    Status(String),
    Timeout(String),
    Body(String),
    Other(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Chat Client error: {:?}", self)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
