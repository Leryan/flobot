use crate::models::{Post, Event};
use crate::middleware::Middleware;

pub struct Instance {
    middlewares: Vec<Box<dyn Middleware>>,
    post_handlers: Vec<Post>
}

impl Instance {
    pub fn process(&self, event: Event) {

    }
}