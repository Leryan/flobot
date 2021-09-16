use crate::client;
use crate::models::GenericEvent;
use std::convert::From;

#[derive(Debug)]
pub enum Error {
    Client(String),
}

pub enum Continue {
    No,
    Yes(GenericEvent),
}

type Result = std::result::Result<Continue, Error>;

impl From<client::Error> for Error {
    fn from(e: client::Error) -> Self {
        Self::Client(format!("{:?}", e))
    }
}

pub trait Middleware {
    fn process(&mut self, event: GenericEvent) -> Result;
}

pub struct Debug {
    name: String,
}

impl Debug {
    pub fn new(name: &str) -> Self {
        Self { name: String::from(name) }
    }
}

impl Middleware for Debug {
    fn process(&mut self, event: GenericEvent) -> Result {
        println!("middleware {:?} -> {:?}", self.name, event);
        Ok(Continue::Yes(event))
    }
}

pub struct IgnoreSelf {
    my_id: String,
}

impl IgnoreSelf {
    pub fn new(my_id: String) -> Self {
        Self { my_id }
    }
}

impl Middleware for IgnoreSelf {
    fn process(&mut self, event: GenericEvent) -> Result {
        match event {
            GenericEvent::Post(post) => {
                if post.user_id == self.my_id {
                    Ok(Continue::No)
                } else {
                    Ok(Continue::Yes(GenericEvent::Post(post)))
                }
            }
            _ => Ok(Continue::Yes(event)),
        }
    }
}
