use crate::client::Client;
use crate::client::Error as ClientError;
use crate::handlers::Handler;
use crate::middleware::Continue;
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

fn client_err(ce: crate::client::Error) -> Error {
    Error::Client(ce)
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Instance got a fatal error: {:?}", self)
    }
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

pub type PostHandler<C> = Box<dyn Handler<C, Data = GenericPost>>;

pub struct Instance<C: Client> {
    middlewares: Vec<Box<dyn Middleware<C>>>,
    post_handlers: Vec<PostHandler<C>>,
    client: C,
    helps: std::collections::HashMap<String, String>,
}

impl<C: Client> Instance<C> {
    pub fn new(client: C) -> Self {
        Instance {
            middlewares: Vec::new(),
            post_handlers: Vec::new(),
            client: client,
            helps: std::collections::HashMap::new(),
        }
    }

    pub fn add_middleware(&mut self, middleware: Box<dyn Middleware<C>>) -> &mut Self {
        self.middlewares.push(middleware);
        self
    }

    pub fn add_post_handler(&mut self, handler: PostHandler<C>) -> &mut Self {
        handler.help().and_then(|help| {
            self.helps
                .insert(handler.name().to_string(), help.to_string())
        });
        self.post_handlers.push(handler);
        self
    }

    fn process_middlewares(&mut self, event: GenericEvent) -> Result<Option<GenericEvent>, Error> {
        let refevent = &mut event.clone();

        for middleware in self.middlewares.iter_mut() {
            match middleware.process(refevent, &mut self.client)? {
                Continue::Yes => continue,
                Continue::No => {
                    return Ok(None);
                }
            };
        }

        Ok(Some(event))
    }

    fn process_help(&self, post: &GenericPost) -> Result<(), Error> {
        if &post.message == "!help" {
            let mut reply = String::new();
            let mut keys: Vec<String> = self.helps.keys().map(|v| v.clone()).collect();
            keys.sort();
            for key in keys.iter() {
                reply.push_str(&format!("`{}`\n", key));
            }

            return self
                .client
                .send_reply(post.clone(), &reply)
                .map_err(client_err);
        }

        match regex::Regex::new("^!help ([a-zA-Z0-9_-]+).*")
            .unwrap()
            .captures(&post.message)
        {
            Some(captures) => {
                let name = captures.get(1).unwrap().as_str();
                match self.helps.get(name) {
                    Some(m) => self.client.send_reply(post.clone(), m),
                    None => self.client.send_reply(post.clone(), "tutétrompé"),
                }
                .map_err(client_err)
            }
            None => Ok(()),
        }
    }

    fn process_event_post(&mut self, post: GenericPost) -> Result<(), Error> {
        let _ = self.process_help(&post)?;
        for handler in self.post_handlers.iter_mut() {
            let res = handler.handle(post.clone(), &self.client);
            let _ = match res {
                Ok(_) => {}
                Err(e) => match self.client.debug(&format!("error: {:?}", e)) {
                    Ok(_) => {}
                    Err(e) => println!("debug error: {:?}", e),
                },
            };
        }
        Ok(())
    }

    fn process_event(&mut self, event: GenericEvent) -> Result<(), Error> {
        match event {
            GenericEvent::Post(post) => self.process_event_post(post),
            GenericEvent::PostEdited(_edited) => {
                println!("edits are unsupported for now");
                Ok(())
            }
            GenericEvent::Unsupported(_unsupported) => {
                //println!("unsupported event: {:?}", unsupported);
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
