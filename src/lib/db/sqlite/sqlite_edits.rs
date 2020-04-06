use crate::db::models::NewEdit;
use crate::db::schema::edits::dsl as table;
use crate::db::Result;
use crate::models;
use diesel::prelude::*;

impl crate::db::Edits for super::Sqlite {
    fn list(&self, team_id: &str) -> Result<Vec<models::Edit>> {
        return Ok(table::edits
            .filter(table::team_id.eq(team_id))
            .order_by(table::user_id) // user edits first, then team
            .limit(1)
            .load::<models::Edit>(&self.db)?);
    }

    /// Find the first Edit available, sorted by user_id, then team_id, and matching edit parameter.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() {
    /// # use diesel::prelude::*;
    /// # use diesel::SqliteConnection;
    /// # use flobot::db::sqlite::Sqlite;
    /// # use flobot::db::{run_migrations, Edits};
    /// # let conn = SqliteConnection::establish(":memory:").unwrap();
    /// # run_migrations(&conn);
    /// # let s = Sqlite::new(conn);
    /// s.add_team("team", "noedit", "noreplace").unwrap();
    /// s.add_team("team", "edit", "replace").unwrap();
    /// s.add_team("noteam", "edit", "noreplace").unwrap();
    /// let e = s.find("user", "team", "edit").unwrap().unwrap(); // no error, Some(edit)
    ///
    /// assert_eq!("edit", e.edit.as_str());
    /// assert_eq!("replace", e.replace_with_text.unwrap().as_str());
    /// assert_eq!(None, e.replace_with_file);
    /// # }
    /// ```
    fn find(&self, user_id: &str, team_id: &str, edit: &str) -> Result<Option<models::Edit>> {
        let res = table::edits
            .filter(
                table::team_id
                    .eq(team_id)
                    .or(table::user_id.eq(user_id))
                    .and(table::edit.eq(edit.trim())),
            )
            .order_by(table::user_id) // user edits first, then team
            .first::<models::Edit>(&self.db);

        match res {
            Ok(edit) => Ok(Some(edit)),
            Err(diesel::NotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn del_team(&self, team_id: &str, edit: &str) -> Result<()> {
        let filter = table::edits.filter(table::team_id.eq(team_id).and(table::edit.eq(edit)));
        let _ = diesel::delete(filter).execute(&self.db)?;
        Ok(())
    }

    fn add_team(&self, team_id: &str, edit: &str, replace: &str) -> Result<()> {
        let edit_ = NewEdit {
            edit,
            replace_with_text: Some(replace),
            replace_with_file: None,
            team_id: Some(team_id),
            user_id: None,
        };

        let _ = diesel::insert_into(table::edits)
            .values(&edit_)
            .execute(&self.db)?;
        Ok(())
    }
}
