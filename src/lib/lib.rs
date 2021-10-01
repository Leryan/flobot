#[macro_use]
extern crate diesel;

pub mod client;
pub mod conf;
pub mod db;
pub mod handlers;
pub mod instance;
pub mod joke;
pub mod mattermost;
pub mod middleware;
pub mod models;
pub mod pinterest;
pub mod task;
pub mod werewolf;

// https://doc.rust-lang.org/nightly/std/macro.env.html - compile time env
pub const BUILD_GIT_HASH: &'static str = env!("BUILD_GIT_HASH");
