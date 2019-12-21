use crate::models::Event;

pub enum Error {
    Stop,
    Error(String),
}

pub trait Middleware {
    fn process(&self, event: Event) -> Result<(), Error>;
}

pub struct One {}

impl Middleware for One {
    fn process(&self, event: Event) -> Result<(), Error> {
        unimplemented!()
    }
}

pub struct Two{
    name: String
}

impl Two {
    pub fn new(name: String) -> Self {
        Two{
            name: name
        }
    }
}

impl Middleware for Two {
    fn process(&self, event: Event) -> Result<(), Error> {
        unimplemented!()
    }
}