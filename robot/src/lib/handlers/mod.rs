use crate::models::Post;

pub trait Handler {
    type Data;
    fn handle(&self, data: Self::Data);
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

impl Handler for Debug {
    type Data = Post;

    fn handle(&self, data: Post) {
        println!("handler {:?} -> {:?}", self.name, data)
    }
}
