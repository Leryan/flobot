use crate::client::Client;
use crate::models::Post;

pub trait Handler<C> {
    type Data;
    fn handle(&self, data: Self::Data, client: &C);
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
    type Data = Post;

    fn handle(&self, data: Post, _client: &C) {
        println!("handler {:?} -> {:?}", self.name, data)
    }
}
