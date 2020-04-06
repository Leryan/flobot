pub mod models;
use diesel::Connection;
use diesel_migrations;
use std::convert::From;

use crate::models as business_models;

#[cfg(feature = "sqlite")]
pub type DatabaseConnection = diesel::SqliteConnection;
#[cfg(feature = "sqlite")]
pub mod sqlite;

pub mod schema;

#[derive(Debug)]
pub enum Error {
    Database(String),
    Migration(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Migration(e) => write!(f, "Cannot run migrations: {}", e),
            Error::Database(e) => write!(f, "db::Error: {}", e),
        }
    }
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Self {
        Error::Database(e.to_string())
    }
}

impl From<diesel_migrations::RunMigrationsError> for Error {
    fn from(e: diesel_migrations::RunMigrationsError) -> Self {
        Error::Migration(e.to_string())
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

pub fn conn(db_url: &str) -> DatabaseConnection {
    return DatabaseConnection::establish(db_url).expect("db connection");
}

pub fn run_migrations(conn: &DatabaseConnection) -> Result<()> {
    let _ = diesel_migrations::run_pending_migrations(conn)?;
    Ok(())
}
