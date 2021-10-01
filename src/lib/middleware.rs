use crate::client;
use crate::models::Event;
use std::convert::From;

#[derive(Debug)]
pub enum Error {
    Client(String),
}

pub enum Continue {
    No,
    Yes(Event),
}

type Result = std::result::Result<Continue, Error>;

impl From<client::Error> for Error {
    fn from(e: client::Error) -> Self {
        Self::Client(format!("{:?}", e))
    }
}

pub trait Middleware {
    fn process(&mut self, event: Event) -> Result;
    fn name(&self) -> &str;
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
    fn process(&mut self, event: Event) -> Result {
        println!("middleware {:?} -> {:?}", self.name, event);
        Ok(Continue::Yes(event))
    }

    fn name(&self) -> &str {
        "Debug"
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
    fn process(&mut self, event: Event) -> Result {
        match event {
            Event::Post(post) => {
                if post.user_id == self.my_id {
                    Ok(Continue::No)
                } else {
                    Ok(Continue::Yes(Event::Post(post)))
                }
            }
            _ => Ok(Continue::Yes(event)),
        }
    }

    fn name(&self) -> &str {
        "IgnoreSelf"
    }
}
