use crate::models::Event;

pub enum Error {
    Stop,
    Error(String),
}

type Result = std::result::Result<bool, Error>;

pub trait Middleware {
    fn process(&self, event: &mut Event) -> Result;
}

pub struct Debug {}

impl Middleware for Debug {
    fn process(&self, event: &mut Event) -> Result {
        println!("{:?}", event);
        Ok(true)
    }
}
