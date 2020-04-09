use crate::models::*;
use crossbeam::crossbeam_channel::Sender;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum Error {
    Status(String),
    Timeout(String),
    Body(String),
    Other(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Chat Client error: {:?}", self)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait EventClient {
    fn listen(&self, sender: Sender<GenericEvent>);
}

pub trait Client {
    fn my_user_id(&self) -> &str;
    fn send_post(&self, post: GenericPost) -> Result<()>;
    fn send_reaction(&self, post: GenericPost, reaction: &str) -> Result<()>;
    fn send_reply(&self, post: GenericPost, message: &str) -> Result<()>;
    fn send_message(&self, from: GenericPost, message: &str) -> Result<()>;
    fn send_trigger_list(&self, triggers: Vec<Trigger>, from: GenericPost) -> Result<()>;
    fn edit_post_message(&self, post_id: &str, message: &str) -> Result<()>;
    fn notify_startup(&self) -> Result<()>;
    fn unimplemented(&self, post: GenericPost) -> Result<()>;
    fn debug(&self, message: &str) -> Result<()>;
}
