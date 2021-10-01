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
use flobot::joke;
use flobot::mattermost::client::Mattermost;
use flobot::middleware;
use flobot::task::*;
use std::env;
use std::fs;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

embed_migrations!();

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("Launch version {}", flobot::BUILD_GIT_HASH);
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

    println!("run db migrations");
    let conn = db::conn(db_url);
    embedded_migrations::run(&conn)?;

    println!("init client, db, handler, middleware...");

    // BASICS
    let mm_client = Mattermost::new(cfg.clone())?;
    let botdb = Rc::new(dbs::Sqlite::new(conn));
    let mut instance = Instance::new(mm_client.clone());

    // TASKRUNNER
    let mut taskrunner = SequentialTaskRunner::new();
    taskrunner.add(Arc::new(Tick {}));

    // MIDDLEWARES & BASIC HANDLERS
    let ignore_self = middleware::IgnoreSelf::new(mm_client.my_user_id().to_string().clone());
    if flag_debug {
        instance.add_middleware(Box::new(middleware::Debug::new("debug")));
    }
    instance.add_middleware(Box::new(ignore_self));

    let trigger_delay_secs = Duration::from_secs(
        std::env::var("BOT_TRIGGER_DELAY_SECONDS")
            .unwrap_or("0".to_string())
            .parse()
            .unwrap(),
    );
    println!("trigger configured with delay of {} seconds", trigger_delay_secs.as_secs());
    let trigger = handlers::trigger::Trigger::new(Rc::clone(&botdb), mm_client.clone(), Tempo::new(), trigger_delay_secs);
    instance.add_post_handler(Box::new(trigger));

    let edits = handlers::edits::Edit::new(Rc::clone(&botdb), mm_client.clone());
    instance.add_post_handler(Box::new(edits));

    // BLAGUES
    let mut jokeproviders: Vec<flobot::joke::Provider> = vec![
        Arc::new(joke::BadJokes::new()),
        Arc::new(joke::Sqlite::new(rand::thread_rng(), Rc::clone(&botdb))),
    ];
    if let Ok(token) = env::var("BOT_BLAGUESAPI_TOKEN") {
        let blaguesapi = joke::BlaguesAPI::new(token.as_str());
        jokeproviders.push(Arc::new(blaguesapi));
    }

    if let Ok(filepath) = env::var("BOT_BLAGUES_URLS") {
        if let Ok(content) = fs::read_to_string(filepath.clone()) {
            let mut urls = vec![];
            for line in content.split("\n") {
                urls.push(line.to_string());
            }

            jokeproviders.push(Arc::new(joke::URLs { urls }));
        } else {
            println!("cannot read jokes from {}", filepath);
        }
    }

    let rnd_blague = joke::SelectProvider::new(rand::thread_rng(), jokeproviders);
    let blague = joke::JokeHandler::new(Rc::clone(&botdb), rnd_blague, mm_client.clone());

    instance.add_post_handler(Box::new(blague));

    // WEREWOLF GAME
    let ww = handlers::ww::Handler::new(mm_client.clone());
    instance.add_post_handler(Box::new(ww));

    // SMS
    if let (Ok(login), Ok(apikey)) = (env::var("BOT_OCTOPUSH_LOGIN"), env::var("BOT_OCTOPUSH_APIKEY")) {
        let smsprov = handlers::sms::Octopush::new(&login, &apikey);
        let sms = handlers::sms::SMS::new(smsprov, Rc::clone(&botdb), mm_client.clone());
        instance.add_post_handler(Box::new(sms));
    }

    // METEO
    if let (Ok(cities), Ok(channel)) = (env::var("BOT_METEO_CITIES"), env::var("BOT_METEO_ON_CHANNEL_ID")) {
        let cities = cities.split(',').map(|p| p.to_string()).collect();
        println!(
            "exec meteo in {:?}",
            taskrunner.add(Arc::new(Meteo::new(cities, mm_client.clone(), &channel)))
        );
    }

    // INSTANCE
    println!("launch bot!");

    // RUN FOREVER
    let (sender, receiver) = unbounded();
    let wg = WaitGroup::new();
    {
        let wg = wg.clone();
        thread::spawn(move || {
            println!("launch client thread");
            mm.listen(sender);
            println!("client thread returned");
            drop(wg);
        });
    }

    let taskrunner = Arc::new(taskrunner);
    {
        let taskrunner = taskrunner.clone();
        let wg = wg.clone();
        thread::spawn(move || {
            println!("launch task runner");
            taskrunner.run_forever();
            println!("task runner returned");
            drop(wg);
        });
    }

    if let Err(e) = instance.run(receiver.clone()) {
        println!("instance returned: {:?}", e);
    }

    println!("stopping task runner");
    taskrunner.stop();
    println!("waiting for threads to stop");
    wg.wait();
    drop(botdb);

    Ok(())
}
