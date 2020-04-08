use diesel::SqliteConnection;

pub struct Sqlite {
    db: SqliteConnection,
}

impl Sqlite {
    pub fn new(db: SqliteConnection) -> Self {
        Self { db: db }
    }
}

mod sqlite_blague;
mod sqlite_edits;
mod sqlite_trigger;
