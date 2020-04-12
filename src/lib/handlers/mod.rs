use crate::client::Client;
use crate::client::Error as ClientError;
use crate::db;
use crate::models::GenericPost;
use std::convert::From;

#[derive(Debug)]
pub enum Error {
    Database(String),
    Timeout(String),
    Status(String),
    Other(String),
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

impl From<ClientError> for Error {
    fn from(e: ClientError) -> Self {
        match e {
            ClientError::Timeout(e) => Error::Timeout(e.to_string()),
            ClientError::Other(e) => Error::Other(e.to_string()),
            ClientError::Status(e) => Error::Status(e.to_string()),
            ClientError::Body(e) => Error::Other(e.to_string()),
        }
    }
}

pub type Result = std::result::Result<(), Error>;

pub trait Handler<C> {
    type Data;
    fn name(&self) -> &str;
    fn help(&self) -> Option<&str>;
    fn handle(&mut self, data: Self::Data, client: &C) -> Result;
}

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

impl<C: Client> Handler<C> for Debug {
    type Data = GenericPost;

    fn name(&self) -> &str {
        "debug"
    }
    fn help(&self) -> Option<&str> {
        None
    }

    fn handle(&mut self, data: GenericPost, _client: &C) -> Result {
        println!("handler {:?} -> {:?}", self.name, data);
        Ok(())
    }
}

pub mod blague;
pub mod edits;
pub mod trigger;
