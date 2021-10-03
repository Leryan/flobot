use crate::client;
use crate::models::Post;
use std::convert::From;

#[derive(Debug)]
pub enum Error {
    Database(String),
    Timeout(String),
    Status(String),
    Other(String),
}

impl From<client::Error> for Error {
    fn from(e: client::Error) -> Self {
        match e {
            client::Error::Timeout(e) => Error::Timeout(e.to_string()),
            client::Error::Other(e) => Error::Other(e.to_string()),
            client::Error::Status(e) => Error::Status(e.to_string()),
            client::Error::Body(e) => Error::Other(e.to_string()),
        }
    }
}

pub type Result = std::result::Result<(), Error>;

/// Handle events after they have been through middleware.
/// Although Data suggest it is possible to support different types of
/// event, only Post are supported currently.
pub trait Handler {
    type Data;
    fn name(&self) -> String;
    fn help(&self) -> Option<String>;
    fn handle(&self, data: &Self::Data) -> Result;
}

/// DO NOT USE IN PRODUCTION: Debug handler will PRINT ALL MESSAGES.
pub struct Debug {
    name: String,
}

impl Debug {
    pub fn new(name: &str) -> Self {
        Debug {
            name: String::from(name),
        }
    }
}

impl Handler for Debug {
    type Data = Post;

    fn name(&self) -> String {
        "debug".into()
    }
    fn help(&self) -> Option<String> {
        None
    }

    fn handle(&self, post: &Post) -> Result {
        println!("debug handler {:?} -> {:?}", self.name, post);
        Ok(())
    }
}
