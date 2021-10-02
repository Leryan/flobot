use crate::db::models::NewBlague;
use crate::db::schema::blague::dsl as table;
use crate::db::Result;
use crate::models::Blague;
use diesel::prelude::*;

impl crate::db::Joke for super::Sqlite {
    fn pick(&self, team_id: &str, relnum: u64) -> Result<Option<Blague>> {
        let filter = table::blague
            .filter(table::team_id.eq(team_id))
            .offset(relnum as i64);
        match filter.first(&*self.db.lock().unwrap()) {
            Ok(b) => Ok(Some(b)),
            Err(e) => match e {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(e.into()),
            },
        }
    }

    fn count(&self, team_id: &str) -> Result<u64> {
        let filter = table::blague.filter(table::team_id.eq(team_id));
        let res: i64 = filter
            .select(diesel::dsl::count_star())
            .first(&*self.db.lock().unwrap())?;
        Ok(res as u64)
    }

    fn list(&self, team_id: &str) -> Result<Vec<Blague>> {
        return Ok(table::blague
            .filter(table::team_id.eq(team_id))
            .order_by(table::id.asc())
            .load::<Blague>(&*self.db.lock().unwrap())?);
    }

    fn del(&self, team_id: &str, id: i32) -> Result<()> {
        let filter =
            table::blague.filter(table::team_id.eq(team_id).and(table::id.eq(id)));
        let _ = diesel::delete(filter).execute(&*self.db.lock().unwrap())?;
        Ok(())
    }

    fn add(&self, team_id: &str, text: &str) -> Result<()> {
        let new_blague = NewBlague {
            team_id: team_id,
            text: text,
        };
        let _ = diesel::insert_into(table::blague)
            .values(&new_blague)
            .execute(&*self.db.lock().unwrap())?;
        Ok(())
    }
}
