use crate::client;
use crate::handler::{Handler, Result as HandlerResult};
use crate::middleware::Continue;
use crate::middleware::Error as MiddlewareError;
use crate::middleware::Middleware as MMiddleware;
use crate::models::{Event, Post, StatusCode, StatusError};
use regex::Regex;
use std::convert::From;
use std::sync::mpsc::Receiver;

#[derive(Debug)]
pub enum Error {
    // FIXME: strip down to Fatal and Error
    Other(String),
    Middleware(MiddlewareError),
    Processing(String),
    Client(client::Error),
    Consumer(String),
    Status(String),
}

fn client_err(ce: client::Error) -> Error {
    Error::Client(ce)
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Instance got a fatal error: {:?}", self)
    }
}

impl From<client::Error> for Error {
    fn from(e: client::Error) -> Self {
        Error::Client(e)
    }
}

impl From<MiddlewareError> for Error {
    fn from(e: MiddlewareError) -> Self {
        Error::Middleware(e)
    }
}

pub type PostHandler = Box<dyn Handler<Data = Post> + Send + Sync>;
pub type Middleware = Box<dyn MMiddleware + Send + Sync>;

pub struct MutexedPostHandler<PH> {
    handler: std::sync::Mutex<PH>,
}

impl<PH> MutexedPostHandler<PH> {
    pub fn from(ph: PH) -> Self {
        Self {
            handler: std::sync::Mutex::new(ph),
        }
    }
}

impl<PH: Handler> Handler for MutexedPostHandler<PH> {
    type Data = PH::Data;

    fn name(&self) -> String {
        self.handler.lock().unwrap().name()
    }

    fn help(&self) -> Option<String> {
        self.handler.lock().unwrap().help()
    }

    fn handle(&self, data: &PH::Data) -> HandlerResult {
        self.handler.lock().unwrap().handle(data)
    }
}

pub struct Instance<C> {
    middlewares: Vec<Middleware>,
    post_handlers: Vec<PostHandler>,
    helps: std::collections::HashMap<String, String>,
    client: C,
}

impl<C: client::Sender + client::Notifier> Instance<C> {
    pub fn new(client: C) -> Self {
        Instance {
            middlewares: Vec::new(),
            post_handlers: Vec::new(),
            helps: std::collections::HashMap::new(),
            client,
        }
    }

    pub fn add_middleware(&mut self, middleware: Middleware) -> &mut Self {
        self.middlewares.push(middleware);
        self
    }

    pub fn add_post_handler(&mut self, handler: PostHandler) -> &mut Self {
        handler.help().and_then(|help| {
            self.helps
                .insert(handler.name().to_string(), help.to_string())
        });
        self.post_handlers.push(handler);
        self
    }

    fn process_middlewares(&self, event: &mut Event) -> Result<Continue, Error> {
        for middleware in self.middlewares.iter() {
            match middleware.process(event)? {
                Continue::Yes => {}
                Continue::No => return Ok(Continue::No),
            };
        }

        Ok(Continue::Yes)
    }

    fn process_help(&self, post: &Post) -> Result<(), Error> {
        if &post.message == "!help" {
            let mut reply = String::new();
            let mut keys: Vec<String> = self.helps.keys().map(|v| v.clone()).collect();
            keys.sort();
            for key in keys.iter() {
                reply.push_str(&format!("`{}`\n", key));
            }

            return self.client.reply(post, &reply).map_err(client_err);
        }

        match Regex::new(r"^!help[\s]+([a-zA-Z0-9_-]+).*")
            .unwrap()
            .captures(&post.message)
        {
            Some(captures) => {
                let name = captures.get(1).unwrap().as_str();
                match self.helps.get(name) {
                    Some(m) => self.client.reply(post, m),
                    None => self.client.reply(post, "tutétrompé"),
                }
                .map_err(client_err)
            }
            None => Ok(()),
        }
    }

    fn process_event_post(&self, post: &Post) -> Result<(), Error> {
        let _ = self.process_help(post)?;
        for handler in self.post_handlers.iter() {
            let res = handler.handle(post);
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

    fn process_event(&self, event: &Event) -> Result<(), Error> {
        match event {
            Event::Post(post) => self.process_event_post(post),
            Event::PostEdited(_edited) => {
                println!("edits are unsupported for now");
                Ok(())
            }
            Event::Unsupported(_unsupported) => {
                //println!("unsupported event: {:?}", unsupported);
                Ok(())
            }
            Event::Hello(hello) => {
                println!("hello server {:?}", hello.server_string);
                Ok(())
            }
            Event::Status(status) => match status.code {
                StatusCode::OK => Ok(()),
                StatusCode::Error => Err(Error::Status(
                    status
                        .error
                        .as_ref()
                        .unwrap_or(&StatusError::new_none())
                        .message
                        .clone(),
                )),
                StatusCode::Unsupported => {
                    println!("unsupported: {:?}", status);
                    Ok(())
                }
                StatusCode::Unknown => Err(Error::Other(
                    status
                        .error
                        .as_ref()
                        .unwrap_or(&StatusError::new_none())
                        .message
                        .clone(),
                )),
            },
            Event::Shutdown => Ok(()), // should not arrive here
        }
    }

    fn process(&self, event: &mut Event) -> Result<(), Error> {
        let res = self.process_middlewares(event)?;
        match res {
            Continue::Yes => self.process_event(event),
            Continue::No => Ok(()),
        }
    }

    pub fn run(&self, receiver: Receiver<Event>) -> Result<(), Error> {
        let mut loaded = String::from("## Loaded middlewares\n");
        for m in self.middlewares.iter() {
            loaded.push_str(&format!(" * `{}`\n", m.name()));
        }
        loaded.push_str("## Loaded post handlers\n");
        for h in self.post_handlers.iter() {
            loaded.push_str(&format!(" * `{}`\n", h.name()));
        }

        let _ = self.client.startup(&loaded)?;

        loop {
            match receiver.recv() {
                Ok(mut event) => match event {
                    Event::Shutdown => return Ok(()),
                    _ => self.process(&mut event)?,
                },
                Err(rte) => {
                    return Err(Error::Consumer(format!(
                        "receiving channel error: {}",
                        rte.to_string()
                    )))
                }
            };
        }
    }
}
