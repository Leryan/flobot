use crate::client::Client;
use crate::models::Event;

type Result = std::result::Result<bool, String>;

pub trait Middleware<C: Client> {
    fn process(&self, event: &mut Event, client: &C) -> Result;
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

impl<C: Client> Middleware<C> for Debug {
    fn process(&self, event: &mut Event, _client: &C) -> Result {
        println!("middleware {:?} -> {:?}", self.name, event);
        Ok(true)
    }
}
