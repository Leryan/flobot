use crossbeam::crossbeam_channel::unbounded;
use crossbeam::sync::WaitGroup;
use diesel::Connection;
use diesel::SqliteConnection;
use diesel_migrations;
use dotenv;
use flobot::client::mattermost::Mattermost;
use flobot::client::*;
use flobot::conf::Conf;
use flobot::handlers;
use flobot::instance::Instance;
use flobot::middleware;
use std::thread;

fn db_conn(db_url: &str) -> SqliteConnection {
    return SqliteConnection::establish(db_url).expect("db connection");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::from_filename("flobot.env").ok();
    let cfg = Conf::new().expect("cfg err");
    let db_url = cfg.db_url.as_str();

    let (sender, receiver) = unbounded();
    let wg = WaitGroup::new();

    {
        let wg = wg.clone();
        let cfg = cfg.clone();
        thread::spawn(move || {
            println!("launch client thread");
            Mattermost::new(cfg).listen(sender);
            drop(wg);
        });
    }

    println!("run db migrations");
    diesel_migrations::run_pending_migrations(&db_conn(db_url))?;

    println!("launch bot!");
    Instance::new(Mattermost::new(cfg.clone()))
        //.add_middleware(Box::new(middleware::Debug::new("debug")))
        .add_middleware(Box::new(middleware::IgnoreSelf::new()))
        .add_post_handler(Box::new(handlers::Trigger::new(db_conn(db_url))))
        .run(receiver.clone())?;

    println!("waiting for listener to stop");
    wg.wait();

    Ok(())
}
