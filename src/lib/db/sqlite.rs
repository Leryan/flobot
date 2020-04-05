use super::models as dbmodel;
use super::{Edits, Result, Trigger};
use crate::diesel;
use crate::diesel::prelude::*;
use crate::models;

pub struct Sqlite {
    db: diesel::SqliteConnection,
}

impl Sqlite {
    pub fn new(db: diesel::SqliteConnection) -> Self {
        Self { db: db }
    }
}

impl Trigger for Sqlite {
    fn list(&self, team_id_: &str) -> Result<Vec<models::Trigger>> {
        use crate::schema::trigger::dsl::*;
        return Ok(trigger
            .filter(team_id.eq(team_id_))
            .order(triggered_by.asc())
            .load::<models::Trigger>(&self.db)?);
    }

    fn search(&self, team_id_: &str) -> Result<Vec<models::Trigger>> {
        use crate::schema::trigger::dsl::*;
        Ok(trigger
            .filter(team_id.eq(team_id_))
            .order_by(text_) // emojis first -> all emoji triggers processed first, then text
            .load::<models::Trigger>(&self.db)?)
    }

    fn add_text(&self, team_id: &str, trigger_: &str, text_: &str) -> Result<()> {
        use crate::schema::trigger;

        let new_trigger = dbmodel::NewTrigger {
            triggered_by: trigger_,
            emoji: None,
            text_: Some(text_),
            team_id: team_id,
        };

        let _ = diesel::insert_into(trigger::table)
            .values(&new_trigger)
            .execute(&self.db)?;
        Ok(())
    }

    fn add_emoji(&self, team_id: &str, trigger_: &str, emoji_: &str) -> Result<()> {
        use crate::schema::trigger;

        let new_trigger = dbmodel::NewTrigger {
            triggered_by: trigger_,
            emoji: Some(emoji_),
            text_: None,
            team_id: team_id,
        };

        let _ = diesel::insert_into(trigger::table)
            .values(&new_trigger)
            .execute(&self.db)?;
        Ok(())
    }

    fn del(&self, team_id_: &str, trigger_: &str) -> Result<()> {
        use crate::schema::trigger::dsl::*;
        let filter = triggered_by.eq(trigger_).and(team_id.eq(team_id_));
        let _ = diesel::delete(trigger.filter(filter)).execute(&self.db)?;
        Ok(())
    }
}

impl Edits for Sqlite {
    fn list(&self, team_id_: &str) -> Result<Vec<models::Edit>> {
        use crate::schema::edits::dsl::*;
        return Ok(edits
            .filter(team_id.eq(team_id_))
            .order_by(user_id) // user edits first, then team
            .limit(1)
            .load::<models::Edit>(&self.db)?);
    }

    fn find(&self, user_id_: &str, team_id_: &str, edit_: &str) -> Result<Option<models::Edit>> {
        use crate::schema::edits::dsl::*;
        let res = edits
            .filter(
                team_id
                    .eq(team_id_)
                    .or(user_id.eq(user_id_))
                    .and(edit.eq(edit_.trim())),
            )
            .order_by(user_id) // user edits first, then team
            .limit(1)
            .load::<models::Edit>(&self.db)?;

        if res.len() == 1 {
            return Ok(Some(res[0].clone()));
        }

        Ok(None)
    }

    fn del_team(&self, team_id_: &str, edit_: &str) -> Result<()> {
        use crate::schema::edits::dsl::*;

        let filter = edits.filter(team_id.eq(team_id_).and(edit.eq(edit_)));
        let _ = diesel::delete(filter).execute(&self.db)?;
        Ok(())
    }

    fn add_team(&self, team_id_: &str, search: &str, replace: &str) -> Result<()> {
        use crate::schema::edits::dsl::*;

        let edit_ = dbmodel::NewEdit {
            edit: search,
            replace_with_text: Some(replace),
            replace_with_file: None,
            team_id: Some(team_id_),
            user_id: None,
        };

        let _ = diesel::insert_into(edits)
            .values(&edit_)
            .execute(&self.db)?;
        Ok(())
    }
}
