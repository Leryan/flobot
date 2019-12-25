use crate::handlers::Handler;
use crate::middleware::Middleware;
use crate::models::{Event, Post, StatusCode, StatusError};
use crossbeam::crossbeam_channel::Receiver;

#[derive(Debug)]
pub enum ErrorCode {
    Unknown,
    Middleware,
    App,
}

#[derive(Debug)]
pub struct Error {
    code: ErrorCode,
    message: String,
}

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

    fn process_middlewares(&self, event: Event) -> Option<Result<Event, Error>> {
        let event = &mut event.clone();

        for middleware in self.middlewares.iter() {
            match middleware.process(event) {
                Ok(cont) => match cont {
                    false => return None,
                    true => continue,
                },
                Err(e) => {
                    return Some(Err(Error {
                        code: ErrorCode::Middleware,
                        message: e,
                    }))
                }
            };
        }

        Some(Ok(event.clone()))
    }

    fn process_event(&self, event: Event) -> Result<(), Error> {
        match event {
            Event::Post(post) => {
                self.post_handlers
                    .iter()
                    .for_each(|handler| handler.handle(post.clone()));
                Ok(())
            }
            Event::Unsupported(unsupported) => {
                println!("unsupported event: {:?}", unsupported);
                Ok(())
            }
            Event::Status(apperror) => match apperror.code {
                StatusCode::OK => Ok(()),
                StatusCode::Error => Err(Error {
                    code: ErrorCode::App,
                    message: apperror.error.unwrap_or(StatusError::new_none()).message,
                }),
                StatusCode::Unsupported => {
                    println!("unsupported: {:?}", apperror);
                    Ok(())
                }
                StatusCode::Unknown => Err(Error {
                    code: ErrorCode::Unknown,
                    message: apperror.error.unwrap_or(StatusError::new_none()).message,
                }),
            },
        }
    }

    fn process(&self, event: Event) -> Result<(), Error> {
        match self.process_middlewares(event) {
            Some(res) => match res {
                Ok(event) => self.process_event(event),
                Err(e) => Err(e),
            },
            None => Ok(()),
        }
    }

    pub fn run(&self, receiver: Receiver<Event>) {
        loop {
            match receiver.recv() {
                Ok(e) => match self.process(e) {
                    Err(e) => {
                        println!("process error: {:?}", e);
                        return;
                    }
                    Ok(_) => {
                        continue;
                    }
                },
                Err(e) => {
                    println!("recv error: {:?}", e);
                    return;
                }
            }
        }
    }
}
