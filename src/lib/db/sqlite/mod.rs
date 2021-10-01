use diesel::SqliteConnection;

pub struct Sqlite {
    db: SqliteConnection,
}

impl Sqlite {
    pub fn new(db: SqliteConnection) -> Self {
        Self { db: db }
    }
}

mod joke;
mod edits;
mod sms;
mod trigger;
