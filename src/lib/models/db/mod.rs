use crate::schema::edits;
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

#[derive(Debug, Queryable)]
pub struct Edit {
    pub id: i32,
    pub edit: String,
    pub team_id: Option<String>,
    pub user_id: Option<String>,
    pub replace_with_text: Option<String>,
    pub replace_with_file: Option<String>,
}

#[derive(Insertable)]
#[table_name = "edits"]
pub struct NewEdit<'a> {
    pub edit: &'a str,
    pub team_id: Option<&'a str>,
    pub user_id: Option<&'a str>,
    pub replace_with_text: Option<&'a str>,
    pub replace_with_file: Option<&'a str>,
}
