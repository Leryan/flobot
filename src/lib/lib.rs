#[macro_use]
extern crate diesel;

pub mod client;
pub mod conf;
pub mod db;
pub mod handlers;
pub mod instance;
pub mod mattermost;
pub mod middleware;
pub mod models;
pub mod werewolf;

pub const BUILD_GIT_HASH: &'static str = env!("BUILD_GIT_HASH");
