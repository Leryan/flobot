use crossbeam::crossbeam_channel::unbounded;
use crossbeam::sync::WaitGroup;
#[macro_use]
extern crate diesel_migrations;
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
use std::env;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

embed_migrations!();

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let cli_args: Vec<String> = env::args().collect();
    let mut flag_debug = false;
    println!("Launched with command line arguments: {:?}", cli_args);
    for cli_arg in cli_args {
        if cli_arg.eq("--debug") {
            flag_debug = true;
        }
    }

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
    embedded_migrations::run(&conn)?;

    println!("init client, db, handler, middleware...");
    let mm_client = Rc::new(Mattermost::new(cfg.clone())?);
    let botdb = Rc::new(dbs::Sqlite::new(conn));
    let tempo = Tempo::new();
    let ignore_self = middleware::IgnoreSelf::new(mm_client.my_user_id().to_string().clone());
    let trigger = handlers::trigger::Trigger::new(
        Rc::clone(&botdb),
        Rc::clone(&mm_client),
        tempo.clone(),
        Duration::from_secs(120),
    );
    let edits = handlers::edits::Edit::new(Rc::clone(&botdb), Rc::clone(&mm_client));
    let remote_blague = db::remote::blague::BadJokes::new();
    let remote_sqlite = db::remote::blague::Sqlite::new(rand::thread_rng(), Rc::clone(&botdb));
    let rnd_blague = db::remote::blague::Select::new(
        rand::thread_rng(),
        Box::new(remote_blague),
        Box::new(remote_sqlite),
    );
    let blague =
        handlers::blague::Blague::new(Rc::clone(&botdb), rnd_blague, Rc::clone(&mm_client));
    let ww = handlers::ww::WW::new(Rc::clone(&mm_client));
    let smsprov = handlers::sms::Octopush::new(
        env::var("BOT_OCTOPUSH_LOGIN")
            .unwrap_or("".to_string())
            .as_str(),
        env::var("BOT_OCTOPUSH_APIKEY")
            .unwrap_or("".to_string())
            .as_str(),
    );
    let sms = handlers::sms::SMS::new(smsprov, Rc::clone(&botdb), Rc::clone(&mm_client));

    println!("launch bot!");
    let client = Rc::clone(&mm_client);
    let mut instance = Instance::new(client);
    instance.add_middleware(Box::new(ignore_self));
    instance.add_post_handler(Box::new(trigger));
    instance.add_post_handler(Box::new(edits));
    instance.add_post_handler(Box::new(blague));
    instance.add_post_handler(Box::new(ww));
    instance.add_post_handler(Box::new(sms));

    if flag_debug {
        instance.add_middleware(Box::new(middleware::Debug::new("debug")));
    }
    instance.run(receiver.clone())?;

    drop(botdb);
    println!("waiting for listener to stop");
    wg.wait();

    Ok(())
}
