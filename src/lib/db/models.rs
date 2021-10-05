use crate::db::schema::blague;
use crate::db::schema::edits;
use crate::db::schema::sms_contact;
use crate::db::schema::sms_prepare;
use crate::db::schema::trigger;
use diesel::Insertable;

#[derive(Insertable)]
#[table_name = "trigger"]
pub struct NewTrigger<'a> {
    pub triggered_by: &'a str,
    pub emoji: Option<&'a str>,
    pub text_: Option<&'a str>,
    pub team_id: &'a str,
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

#[derive(Insertable)]
#[table_name = "blague"]
pub struct NewJoke<'a> {
    pub team_id: &'a str,
    pub text: &'a str,
}

#[derive(Insertable)]
#[table_name = "sms_contact"]
pub struct NewSMSContact<'a> {
    pub team_id: &'a str,
    pub name: &'a str,
    pub number: &'a str,
    pub last_sending_unixts: &'a i64,
}

#[derive(Insertable)]
#[table_name = "sms_prepare"]
pub struct NewSMSPrepare<'a> {
    pub team_id: &'a str,
    pub sms_contact_id: &'a i32,
    pub trigname: &'a str,
    pub name: &'a str,
    pub text: &'a str,
}

// db
use diesel::Queryable;

#[derive(Debug, Queryable, Clone)]
pub struct Edit {
    pub id: i32,
    pub edit: String,
    pub team_id: Option<String>,
    pub user_id: Option<String>,
    pub replace_with_text: Option<String>,
    pub replace_with_file: Option<String>,
}

#[derive(Debug, Queryable)]
pub struct Trigger {
    pub id: i32,
    pub triggered_by: String,
    pub emoji: Option<String>,
    pub text_: Option<String>,
    pub team_id: String,
}

#[derive(Debug, Queryable)]
pub struct Joke {
    pub id: i32,
    pub team_id: String,
    pub text: String,
}

#[derive(Debug, Queryable)]
pub struct SMSContact {
    pub id: i32,
    pub team_id: String,
    pub name: String,
    pub number: String,
    pub last_sending_unixts: i64,
}

#[derive(Debug, Queryable)]
pub struct SMSPrepare {
    pub id: i32,
    pub team_id: String,
    pub contact_id: i32,
    pub trigname: String,
    pub name: String,
    pub text: String,
}
