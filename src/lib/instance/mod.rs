use crate::client::Client;
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

pub struct Instance<'c, C: Client> {
    middlewares: Vec<Box<dyn Middleware<C>>>,
    post_handlers: Vec<Box<dyn Handler<C, Data = Post>>>,
    client: &'c C,
}

impl<'c, C: Client> Instance<'c, C> {
    pub fn new(client: &'c C) -> Self {
        Instance {
            middlewares: Vec::new(),
            post_handlers: Vec::new(),
            client: client,
        }
    }

    pub fn add_middleware(&mut self, middleware: Box<dyn Middleware<C>>) -> &mut Self {
        self.middlewares.push(middleware);
        self
    }

    pub fn add_post_handler(&mut self, handler: Box<dyn Handler<C, Data = Post>>) -> &mut Self {
        self.post_handlers.push(handler);
        self
    }

    fn process_middlewares(&self, event: Event) -> Result<Option<Event>, Error> {
        let event = &mut event.clone();

        for middleware in self.middlewares.iter() {
            match middleware.process(event, self.client) {
                Ok(cont) => match cont {
                    false => return Ok(None),
                    true => continue,
                },
                Err(e) => {
                    return Err(Error {
                        code: ErrorCode::Middleware,
                        message: e,
                    })
                }
            };
        }

        Ok(Some(event.clone()))
    }

    fn process_event(&self, event: Event) -> Result<(), Error> {
        match event {
            Event::Post(post) => {
                self.post_handlers
                    .iter()
                    .for_each(|handler| handler.handle(post.clone(), self.client));
                Ok(())
            }
            Event::Unsupported(unsupported) => {
                println!("unsupported event: {:?}", unsupported);
                Ok(())
            }
            Event::Status(apperror) => match apperror.code {
                StatusCode::OK => Err(Error {
                    code: ErrorCode::App,
                    message: "".to_string(),
                }),
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
            Ok(res) => match res {
                Some(event) => self.process_event(event),
                None => Ok(()),
            },
            Err(e) => Err(e),
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
