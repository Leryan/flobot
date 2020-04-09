use crate::client::Client;
use crate::client::Error as ClientError;
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

impl From<ClientError> for Error {
    fn from(e: ClientError) -> Self {
        Self::Client(format!("{:?}", e))
    }
}

pub trait Middleware<C: Client> {
    fn process(&mut self, event: GenericEvent, client: &C) -> Result;
}

pub struct Debug {
    name: String,
}

impl Debug {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
        }
    }
}

impl<C: Client> Middleware<C> for Debug {
    fn process(&mut self, event: GenericEvent, _client: &C) -> Result {
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

impl<C: Client> Middleware<C> for IgnoreSelf {
    fn process(&mut self, event: GenericEvent, _client: &C) -> Result {
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
