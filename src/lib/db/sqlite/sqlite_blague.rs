use crate::db::models::NewBlague;
use crate::db::schema::blague::dsl as table;
use crate::db::Result;
use crate::models::Blague;
use diesel::prelude::*;

impl crate::db::Blague for super::Sqlite {
    fn list(&self, team_id: &str) -> Result<Vec<Blague>> {
        return Ok(table::blague
            .filter(table::team_id.eq(team_id))
            .order_by(table::id.asc())
            .load::<Blague>(&self.db)?);
    }

    fn del(&self, team_id: &str, id: i32) -> Result<()> {
        let filter = table::blague.filter(table::team_id.eq(team_id).and(table::id.eq(id)));
        let _ = diesel::delete(filter).execute(&self.db)?;
        Ok(())
    }

    fn add(&self, team_id: &str, text: &str) -> Result<()> {
        let new_blague = NewBlague {
            team_id: team_id,
            text: text,
        };
        let _ = diesel::insert_into(table::blague)
            .values(&new_blague)
            .execute(&self.db)?;
        Ok(())
    }
}
