use crate::client::Client;
use crate::handlers::Handler;
use crate::middleware::Middleware;
use crate::models::{GenericEvent, GenericPost, StatusCode, StatusError};
use crossbeam::crossbeam_channel::{Receiver, RecvTimeoutError};
use std::fmt;
use std::time::Duration;

#[derive(Debug)]
pub enum ErrorCode {
    Other,
    Middleware,
    Processing,
    Client,
}

#[derive(Debug)]
pub struct Error {
    code: ErrorCode,
    message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(code: {:?}, message: {})", self.code, self.message)
    }
}

impl std::error::Error for Error {}

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
            GenericEvent::PostEdited(_edited) => {
                println!("edits are unsupported for now");
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
                    code: ErrorCode::Client,
                    message: status.error.unwrap_or(StatusError::new_none()).message,
                }),
                StatusCode::Unsupported => {
                    println!("unsupported: {:?}", status);
                    Ok(())
                }
                StatusCode::Unknown => Err(Error {
                    code: ErrorCode::Other,
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

    pub fn run(&mut self, receiver: Receiver<GenericEvent>) -> Result<(), Error> {
        self.client.notify_startup();
        loop {
            match receiver.recv_timeout(Duration::from_secs(5)) {
                Ok(e) => match self.process(e) {
                    Err(e) => {
                        return Err(Error {
                            code: ErrorCode::Processing,
                            message: format!("processing error: {:?}", e),
                        });
                    }
                    Ok(_) => {
                        continue;
                    }
                },
                Err(rte) => match rte {
                    RecvTimeoutError::Timeout => {}
                    RecvTimeoutError::Disconnected => {
                        return Err(Error {
                            code: ErrorCode::Client,
                            message: format!("receiving channel closed"),
                        });
                    }
                },
            }
        }
    }
}
