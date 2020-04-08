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
    Yes,
}

type Result = std::result::Result<Continue, Error>;

impl From<ClientError> for Error {
    fn from(e: ClientError) -> Self {
        Self::Client(format!("{:?}", e))
    }
}

pub trait Middleware<C: Client> {
    fn process(&mut self, event: &mut GenericEvent, client: &mut C) -> Result;
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
    fn process(&mut self, event: &mut GenericEvent, _client: &mut C) -> Result {
        println!("middleware {:?} -> {:?}", self.name, event);
        Ok(Continue::Yes)
    }
}

pub struct IgnoreSelf {
    my_user_id: String,
}

impl IgnoreSelf {
    pub fn new() -> Self {
        Self {
            my_user_id: "".to_string(),
        }
    }
}

impl<C: Client> Middleware<C> for IgnoreSelf {
    fn process(&mut self, event: &mut GenericEvent, client: &mut C) -> Result {
        match event {
            GenericEvent::Hello(hello) => {
                self.my_user_id = hello.my_user_id.clone();
                client.set_my_user_id(&self.my_user_id)?;
                println!("updated my true self {}", self.my_user_id);
                Ok(Continue::Yes)
            }
            GenericEvent::Post(post) => {
                if post.user_id == self.my_user_id {
                    Ok(Continue::No)
                } else {
                    Ok(Continue::Yes)
                }
            }
            _ => Ok(Continue::Yes),
        }
    }
}
