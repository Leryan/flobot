use diesel::SqliteConnection;

pub struct Sqlite {
    db: SqliteConnection,
}

impl Sqlite {
    pub fn new(db: SqliteConnection) -> Self {
        Self { db: db }
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
