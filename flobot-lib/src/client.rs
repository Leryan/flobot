use crate::models::*;
use std::convert::From;

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Error {
        if e.is_timeout() {
            return Error::Timeout(e.to_string());
        }

        if e.is_status() {
            return Error::Status(e.status().unwrap().as_u16() as u64);
        }

        if e.is_builder() {
            return Error::Body(e.to_string());
        }

        Error::Other(e.to_string())
    }
}

/// Error for HTTP/WebSocket clients. Automatic conversion is implemented for
/// the reqwest::Error type.
#[derive(Debug)]
pub enum Error {
    Status(u64),
    Timeout(String),
    Body(String),
    Other(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "client error: {:?}", self)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Send something to the backend.
pub trait Sender {
    fn post(&self, post: &Post) -> Result<()>;
    fn reaction(&self, post: &Post, reaction: &str) -> Result<()>;
    fn reply(&self, post: &Post, message: &str) -> Result<()>;
}

pub trait Editor {
    /// edit an existing post so it contains message instead.
    fn edit(&self, post: &Post, message: &str) -> Result<()>;
}

pub trait Channel {
    /// Creates a private channel and returns the room id to be used as channel_id in a GenericPost
    fn create_private(
        &self,
        team_id: &str,
        name: &str,
        users: &Vec<String>,
    ) -> Result<String>;
    /// Archive given channel.
    fn archive(&self, channel_id: &str) -> Result<()>;
}

pub trait Getter {
    fn my_user_id(&self) -> &str;
    fn users_by_ids(&self, ids: Vec<&str>) -> Result<Vec<User>>;
}

/// A Notifier implementation should only send messages to the debugging channel.
/// See conf::Conf.
pub trait Notifier {
    fn startup(&self, message: &str) -> Result<()>;
    fn debug(&self, message: &str) -> Result<()>;
    fn error(&self, message: &str) -> Result<()>;
    fn required_action(&self, message: &str) -> Result<()>;
}
