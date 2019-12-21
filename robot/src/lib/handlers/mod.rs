use crate::models::Post;

pub trait Handler {
    type Data;
    fn handle(&self, data: Self::Data);
}

pub struct Debug {}

impl Handler for Debug {
    type Data = Post;

    fn handle(&self, data: Post) {
        println!("{:?}", data)
    }
}
