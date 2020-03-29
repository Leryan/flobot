use crate::client::Client;
use crate::handlers::Handler;
use crate::middleware::Middleware;
use crate::models::{GenericEvent, GenericPost, StatusCode, StatusError};
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

pub struct Instance<C: Client> {
    middlewares: Vec<Box<dyn Middleware<C>>>,
    post_handlers: Vec<Box<dyn Handler<C, Data = GenericPost>>>,
    client: C,
}

impl<C: Client> Instance<C> {
    pub fn new(client: C) -> Self {
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

    pub fn add_post_handler(
        &mut self,
        handler: Box<dyn Handler<C, Data = GenericPost>>,
    ) -> &mut Self {
        self.post_handlers.push(handler);
        self
    }

    fn process_middlewares(&mut self, event: GenericEvent) -> Result<Option<GenericEvent>, Error> {
        let event = &mut event.clone();

        for middleware in self.middlewares.iter_mut() {
            match middleware.process(event, &mut self.client) {
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

    fn process_event(&self, event: GenericEvent) -> Result<(), Error> {
        match event {
            GenericEvent::Post(post) => {
                self.post_handlers
                    .iter()
                    .for_each(|handler| handler.handle(post.clone(), &self.client));
                Ok(())
            }
            GenericEvent::Unsupported(unsupported) => {
                println!("unsupported event: {:?}", unsupported);
                Ok(())
            }
            GenericEvent::Hello(hello) => {
                println!("hello server {:?}", hello.server_string);
                Ok(())
            }
            GenericEvent::Status(status) => match status.code {
                StatusCode::OK => Ok(()),
                StatusCode::Error => Err(Error {
                    code: ErrorCode::App,
                    message: status.error.unwrap_or(StatusError::new_none()).message,
                }),
                StatusCode::Unsupported => {
                    println!("unsupported: {:?}", status);
                    Ok(())
                }
                StatusCode::Unknown => Err(Error {
                    code: ErrorCode::Unknown,
                    message: status.error.unwrap_or(StatusError::new_none()).message,
                }),
            },
        }
    }

    fn process(&mut self, event: GenericEvent) -> Result<(), Error> {
        match self.process_middlewares(event) {
            Ok(res) => match res {
                Some(event) => self.process_event(event),
                None => Ok(()),
            },
            Err(e) => Err(e),
        }
    }

    pub fn run(&mut self, receiver: Receiver<GenericEvent>) -> Result<(), String> {
        loop {
            match receiver.recv() {
                Ok(e) => match self.process(e) {
                    Err(e) => {
                        return Err(String::from(format!("processing error: {:?}", e)));
                    }
                    Ok(_) => {
                        continue;
                    }
                },
                Err(e) => {
                    return Err(String::from(format!("recv error: {:?}", e.to_string())));
                }
            }
        }
    }
}
