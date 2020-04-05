pub mod models;
pub mod sqlite;
use std::convert::From;

use crate::models as business_models;

#[derive(Debug)]
pub enum Error {
    Database(String),
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Self {
        Error::Database(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Trigger {
    fn list(&self, team_id: &str) -> Result<Vec<business_models::Trigger>>;
    fn search(&self, team_id_: &str) -> Result<Vec<business_models::Trigger>>;
    fn add_text(&self, team_id: &str, trigger: &str, text: &str) -> Result<()>;
    fn add_emoji(&self, team_id: &str, trigger: &str, emoji: &str) -> Result<()>;
    fn del(&self, team_id: &str, trigger: &str) -> Result<()>;
}

pub trait Edits {
    fn list(&self, team_id: &str) -> Result<Vec<business_models::Edit>>;
    fn find(
        &self,
        user_id: &str,
        team_id: &str,
        edit: &str,
    ) -> Result<Option<business_models::Edit>>;
    fn del_team(&self, team_id: &str, edit: &str) -> Result<()>;
    fn add_team(&self, team_id: &str, edit: &str, replace: &str) -> Result<()>;
}
