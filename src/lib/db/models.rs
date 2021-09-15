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
pub struct NewBlague<'a> {
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
