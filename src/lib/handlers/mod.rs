use crate::client;
use crate::db;
use crate::models::GenericPost;
use std::convert::From;

#[derive(Debug)]
pub enum Error {
    Database(String),
    Timeout(String),
    Status(String),
    Other(String),
    Reaction(String),
    Reply(String),
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            return Error::Timeout(e.to_string());
        }

        if e.is_status() {
            return Error::Status(e.to_string());
        }

        Error::Other(e.to_string())
    }
}

impl From<db::Error> for Error {
    fn from(e: db::Error) -> Self {
        match e {
            db::Error::Migration(e) => Error::Other(e),
            db::Error::Database(e) => Error::Database(e),
        }
    }
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

pub trait Handler {
    type Data;
    fn name(&self) -> &str;
    fn help(&self) -> Option<&str>;
    fn handle(&self, data: &Self::Data) -> Result;
}

pub struct Debug {
    name: String,
}

impl Debug {
    pub fn new(name: &str) -> Self {
        Debug { name: String::from(name) }
    }
}

impl Handler for Debug {
    type Data = GenericPost;

    fn name(&self) -> &str {
        "debug"
    }
    fn help(&self) -> Option<&str> {
        None
    }

    fn handle(&self, post: &GenericPost) -> Result {
        println!("handler {:?} -> {:?}", self.name, post);
        Ok(())
    }
}

pub mod blague;
pub mod edits;
pub mod sms;
pub mod trigger;
pub mod ww;
