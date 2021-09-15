pub mod blague;
use std::convert::From;

pub enum Error {
    Database(String),
    Client(String),
    NoData(String),
    Other(String),
}

pub type Result = std::result::Result<String, Error>;

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        if e.is_builder() || e.is_status() || e.is_timeout() {
            return Self::Client(e.to_string());
        }

        Self::Other(e.to_string())
    }
}

pub trait Blague {
    fn random(&self, team_id: &str) -> Result;
}
