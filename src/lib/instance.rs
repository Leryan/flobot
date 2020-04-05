use crate::client::Client;
use crate::client::Error as ClientError;
use crate::handlers::Handler;
use crate::middleware::Error as MiddlewareError;
use crate::middleware::Middleware;
use crate::models::{GenericEvent, GenericPost, StatusCode, StatusError};
use crossbeam::crossbeam_channel::{Receiver, RecvTimeoutError};
use std::convert::From;
use std::time::Duration;

#[derive(Debug)]
pub enum Error {
    Other(String),
    Middleware(MiddlewareError),
    Processing(String),
    Client(ClientError),
    Consumer(String),
    Status(String),
}

impl From<ClientError> for Error {
    fn from(e: ClientError) -> Self {
        Error::Client(e)
    }
}

impl From<MiddlewareError> for Error {
    fn from(e: MiddlewareError) -> Self {
        Error::Middleware(e)
    }
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
            match middleware.process(event, &mut self.client)? {
                false => {
                    return Ok(None);
                }
                true => {
                    continue;
                }
            };
        }

        Ok(Some(event.clone()))
    }

    fn process_event(&self, event: GenericEvent) -> Result<(), Error> {
        match event {
            GenericEvent::Post(post) => {
                for handler in self.post_handlers.iter() {
                    let res = handler.handle(post.clone(), &self.client);
                    let _ = match res {
                        Ok(_) => {}
                        Err(e) => match self.client.debug(format!("error: {:?}", e).as_str()) {
                            Ok(_) => {}
                            Err(e) => println!("debug error: {:?}", e),
                        },
                    };
                }
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
                StatusCode::Error => Err(Error::Status(
                    status.error.unwrap_or(StatusError::new_none()).message,
                )),
                StatusCode::Unsupported => {
                    println!("unsupported: {:?}", status);
                    Ok(())
                }
                StatusCode::Unknown => Err(Error::Other(
                    status.error.unwrap_or(StatusError::new_none()).message,
                )),
            },
        }
    }

    fn process(&mut self, event: GenericEvent) -> Result<(), Error> {
        let res = self.process_middlewares(event)?;
        match res {
            Some(event) => self.process_event(event),
            None => Ok(()),
        }
    }

    pub fn run(&mut self, receiver: Receiver<GenericEvent>) -> Result<(), Error> {
        let _ = self.client.notify_startup()?;
        loop {
            match receiver.recv_timeout(Duration::from_secs(5)) {
                Ok(e) => {
                    self.process(e)?;
                }
                Err(rte) => match rte {
                    RecvTimeoutError::Timeout => {}
                    RecvTimeoutError::Disconnected => {
                        return Err(Error::Consumer(format!("receiving channel closed")));
                    }
                },
            };
        }
    }
}
