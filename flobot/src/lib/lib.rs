#[macro_use]
extern crate diesel;

pub mod db;
pub mod edits;
pub mod joke;
pub mod pinterest;
pub mod sms;
pub mod trigger;
pub mod weather;
pub mod werewolf;
pub mod werewolf_game;

use flobot_lib::handler::Error as HandlerError;

impl From<db::Error> for HandlerError {
    fn from(e: db::Error) -> Self {
        match e {
            db::Error::Migration(e) => HandlerError::Other(e),
            db::Error::Database(e) => HandlerError::Database(e),
        }
    }
}
