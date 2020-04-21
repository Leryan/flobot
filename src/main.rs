use crossbeam::crossbeam_channel::unbounded;
use crossbeam::sync::WaitGroup;
use dotenv;
use flobot::client::*;
use flobot::conf::Conf;
use flobot::db;
use flobot::db::sqlite as dbs;
use flobot::db::tempo::Tempo;
use flobot::handlers;
use flobot::instance::Instance;
use flobot::mattermost::Mattermost;
use flobot::middleware;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    dotenv::from_filename("flobot.env").ok();
    let cfg = Conf::new().expect("cfg err");

    let mm = Mattermost::new(cfg.clone())?;

    let db_url: &str = &cfg.db_url;

    let (sender, receiver) = unbounded();
    let wg = WaitGroup::new();

    {
        let wg = wg.clone();
        thread::spawn(move || {
            println!("launch client thread");
            mm.listen(sender);
            drop(wg);
        });
    }

    println!("run db migrations");
    let conn = db::conn(db_url);
    db::run_migrations(&conn)?;

    println!("init client, db, handler, middleware...");
    let client = Rc::new(Mattermost::new(cfg.clone())?);
    let botdb = Rc::new(dbs::Sqlite::new(conn));
    let tempo = Tempo::new();
    let ignore_self = middleware::IgnoreSelf::new(client.my_user_id().to_string().clone());
    let trigger = handlers::trigger::Trigger::new(
        Rc::clone(&botdb),
        Rc::clone(&client),
        tempo.clone(),
        Duration::from_secs(120),
    );
    let edits = handlers::edits::Edit::new(Rc::clone(&botdb), Rc::clone(&client));
    let remote_blague = db::remote::blague::BadJokes::new();
    let remote_sqlite = db::remote::blague::Sqlite::new(rand::thread_rng(), Rc::clone(&botdb));
    let rnd_blague = db::remote::blague::Select::new(
        rand::thread_rng(),
        Box::new(remote_blague),
        Box::new(remote_sqlite),
    );
    let blague = handlers::blague::Blague::new(Rc::clone(&botdb), rnd_blague, Rc::clone(&client));

    println!("launch bot!");
    Instance::new(Rc::clone(&client))
        .add_middleware(Box::new(middleware::Debug::new("debug")))
        .add_middleware(Box::new(ignore_self))
        .add_post_handler(Box::new(trigger))
        .add_post_handler(Box::new(edits))
        .add_post_handler(Box::new(blague))
        .run(receiver.clone())?;

    drop(botdb);
    println!("waiting for listener to stop");
    wg.wait();

    Ok(())
}
