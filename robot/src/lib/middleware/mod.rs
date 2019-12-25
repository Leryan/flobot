use crate::models::Event;

type Result = std::result::Result<bool, String>;

pub trait Middleware {
    fn process(&self, event: &mut Event) -> Result;
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

impl Middleware for Debug {
    fn process(&self, event: &mut Event) -> Result {
        println!("middleware {:?} -> {:?}", self.name, event);
        Ok(true)
    }
}
