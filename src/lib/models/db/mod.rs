use crate::schema::trigger;
use diesel::{Insertable, Queryable};

#[derive(Debug, Queryable)]
pub struct Trigger {
    pub id: i32,
    pub triggered_by: String,
    pub emoji: Option<String>,
    pub text_: Option<String>,
    pub team_id: String,
}

#[derive(Insertable)]
#[table_name = "trigger"]
pub struct NewTrigger<'a> {
    pub triggered_by: &'a str,
    pub emoji: Option<&'a str>,
    pub text_: Option<&'a str>,
    pub team_id: &'a str,
}
