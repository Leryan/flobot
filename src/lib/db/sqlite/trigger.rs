use crate::db::models::{NewTrigger, Trigger};
use crate::db::schema::trigger::dsl as table;
use crate::db::Result;
use diesel::prelude::*;

impl crate::db::Trigger for super::Sqlite {
    fn list(&self, team_id: &str) -> Result<Vec<Trigger>> {
        return Ok(table::trigger
            .filter(table::team_id.eq(team_id))
            .order(table::triggered_by.asc())
            .load::<Trigger>(&*self.db.lock().unwrap())?);
    }

    fn search(&self, team_id: &str) -> Result<Vec<Trigger>> {
        Ok(table::trigger
            .filter(table::team_id.eq(team_id))
            .order_by(table::text_) // emojis first -> all emoji triggers processed first, then text
            .load::<Trigger>(&*self.db.lock().unwrap())?)
    }

    fn add_text(&self, team_id: &str, trigger_: &str, text_: &str) -> Result<()> {
        let new_trigger = NewTrigger {
            triggered_by: trigger_,
            emoji: None,
            text_: Some(text_),
            team_id: team_id,
        };

        let _ = diesel::insert_into(table::trigger)
            .values(&new_trigger)
            .execute(&*self.db.lock().unwrap())?;
        Ok(())
    }

    fn add_emoji(&self, team_id: &str, trigger_: &str, emoji: &str) -> Result<()> {
        let new_trigger = NewTrigger {
            triggered_by: trigger_,
            emoji: Some(emoji),
            text_: None,
            team_id: team_id,
        };

        let _ = diesel::insert_into(table::trigger)
            .values(&new_trigger)
            .execute(&*self.db.lock().unwrap())?;
        Ok(())
    }

    fn del(&self, team_id_: &str, trigger_: &str) -> Result<()> {
        let filter = table::triggered_by
            .eq(trigger_)
            .and(table::team_id.eq(team_id_));
        let _ = diesel::delete(table::trigger.filter(filter))
            .execute(&*self.db.lock().unwrap())?;
        Ok(())
    }
}
