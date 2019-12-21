use crate::handlers::Handler;
use crate::middleware::Middleware;
use crate::models::{Event, Post};

pub struct Instance {
    middlewares: Vec<Box<dyn Middleware>>,
    post_handlers: Vec<Box<dyn Handler<Data = Post>>>,
}

impl Instance {
    pub fn new() -> Self {
        Instance {
            middlewares: Vec::new(),
            post_handlers: Vec::new(),
        }
    }

    pub fn add_middleware(&mut self, middleware: Box<dyn Middleware>) -> &mut Self {
        self.middlewares.push(middleware);
        self
    }

    pub fn add_post_handler(&mut self, handler: Box<dyn Handler<Data = Post>>) -> &mut Self {
        self.post_handlers.push(handler);
        self
    }

    pub fn process(&self, event: Event) {
        let event = &mut event.clone();

        for middleware in self.middlewares.iter() {
            match middleware.process(event) {
                Ok(cont) => match cont {
                    false => return,
                    true => {}
                },
                Err(_) => return,
            }
        }

        let event = event.clone();

        match event {
            Event::Post(post) => self.process_event(post),
        }
    }

    fn process_event(&self, post: Post) {
        for handler in self.post_handlers.iter() {
            handler.handle(post.clone());
        }
    }
}
