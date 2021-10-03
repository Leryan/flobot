pub mod client;
pub mod conf;
pub mod handler;
pub mod instance;
pub mod middleware;
pub mod models;
pub mod task;
pub mod tempo;

// https://doc.rust-lang.org/nightly/std/macro.env.html - compile time env
pub const BUILD_GIT_HASH: &'static str = env!("BUILD_GIT_HASH");
