#[macro_use]
extern crate diesel;

pub mod client;
pub mod conf;
pub mod db;
pub mod handlers;
pub mod joke;
pub mod mattermost;
pub mod pinterest;
pub mod weather;
pub mod werewolf;

// https://doc.rust-lang.org/nightly/std/macro.env.html - compile time env
pub const BUILD_GIT_HASH: &'static str = env!("BUILD_GIT_HASH");

pub trait SendTriggerList {
    fn send_trigger_list(
        &self,
        triggers: Vec<crate::db::models::Trigger>,
        from: &flobot_lib::models::Post,
    ) -> flobot_lib::client::Result<()>;
}
