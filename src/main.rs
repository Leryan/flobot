use crossbeam::crossbeam_channel::unbounded;
use crossbeam::sync::WaitGroup;
#[macro_use]
extern crate diesel_migrations;
use dotenv;
use flobot::client::*;
use flobot::conf::Conf;
use flobot::db;
use flobot::db::remote::{blague as rdb_blague, Blague};
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

    // BASICS
    let mm_client = Rc::new(Mattermost::new(cfg.clone())?);
    let client = Rc::clone(&mm_client);
    let botdb = Rc::new(dbs::Sqlite::new(conn));
    let tempo = Tempo::new();
    let mut instance = Instance::new(client);

    // MIDDLEWARES & BASIC HANDLERS
    let ignore_self = middleware::IgnoreSelf::new(mm_client.my_user_id().to_string().clone());
    if flag_debug {
        instance.add_middleware(Box::new(middleware::Debug::new("debug")));
    }
    instance.add_middleware(Box::new(ignore_self));

    let trigger = handlers::trigger::Trigger::new(Rc::clone(&botdb), Rc::clone(&mm_client), tempo.clone(), Duration::from_secs(120));
    instance.add_post_handler(Box::new(trigger));

    let edits = handlers::edits::Edit::new(Rc::clone(&botdb), Rc::clone(&mm_client));
    instance.add_post_handler(Box::new(edits));

    // BLAGUES
    let mut blague_providers: Vec<Box<dyn Blague>> = vec![
        Box::new(rdb_blague::BadJokes::new()),
        Box::new(rdb_blague::Sqlite::new(rand::thread_rng(), Rc::clone(&botdb))),
    ];
    if let Ok(token) = env::var("BOT_BLAGUESAPI_TOKEN") {
        let blaguesapi = rdb_blague::BlaguesAPI::new(token.as_str());
        blague_providers.push(Box::new(blaguesapi));
    }
    let rnd_blague = rdb_blague::Select::new(rand::thread_rng(), blague_providers);
    let blague = handlers::blague::Blague::new(Rc::clone(&botdb), rnd_blague, Rc::clone(&mm_client));

    instance.add_post_handler(Box::new(blague));

    // WEREWOLF GAME
    let ww = handlers::ww::Handler::new(Rc::clone(&mm_client));
    instance.add_post_handler(Box::new(ww));

    // SMS
    if let (Ok(login), Ok(apikey)) = (env::var("BOT_OCTOPUSH_LOGIN"), env::var("BOT_OCTOPUSH_APIKEY")) {
        let smsprov = handlers::sms::Octopush::new(&login, &apikey);
        let sms = handlers::sms::SMS::new(smsprov, Rc::clone(&botdb), Rc::clone(&mm_client));
        instance.add_post_handler(Box::new(sms));
    }

    // INSTANCE
    println!("launch bot!");

    // RUN FOREVER
    instance.run(receiver.clone())?;

    drop(botdb);
    println!("waiting for listener to stop");
    wg.wait();

    Ok(())
}
