use diesel::SqliteConnection;
use std::sync::Mutex;

pub struct Sqlite {
    db: Mutex<SqliteConnection>,
}

impl Sqlite {
    pub fn new(db: SqliteConnection) -> Self {
        Self { db: Mutex::new(db) }
    }
}

/// new is a shortcut for Sqlite::new
pub fn new(db: SqliteConnection) -> Sqlite {
    Sqlite::new(db)
}

mod edits;
mod joke;
mod sms;
mod trigger;
