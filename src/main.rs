#[macro_use]
extern crate diesel_migrations;
use dotenv;
use flobot::client::*;
use flobot::conf::Conf;
use flobot::db;
use flobot::db::tempo::Tempo;
use flobot::handlers::{
    edits::Edit as HandlerEdit, sms, trigger::Trigger as HandlerTrigger,
    ww::Handler as HandlerWW,
};
use flobot::instance::{Instance, MutexedPostHandler};
use flobot::joke;
use flobot::mattermost::client::Mattermost;
use flobot::middleware;
use flobot::task::*;
use signal_libc::signal::{self, Signal};
use std::env;
use std::fs;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

embed_migrations!();

fn make_jokes_provider(botdb: Arc<db::sqlite::Sqlite>) -> joke::SelectProvider {
    let mut joke_remotes = joke::SelectProvider::new(vec![]);
    joke_remotes.push(Box::new(joke::ProviderBadJokes::new()));
    joke_remotes.push(Box::new(joke::ProviderSQLite::new(botdb)));
    if let Ok(token) = env::var("BOT_BLAGUESAPI_TOKEN") {
        let blaguesapi = joke::ProviderBlaguesAPI::new(&token);
        joke_remotes.push(Box::new(blaguesapi));
    }

    if let Ok(filepath) = env::var("BOT_BLAGUES_URLS") {
        if let Ok(content) = fs::read_to_string(filepath.clone()) {
            let mut urls = vec![];
            for line in content.split("\n") {
                urls.push(line.to_string());
            }

            joke_remotes.push(Box::new(joke::ProviderFile { urls }));
        } else {
            println!("cannot read jokes from {}", filepath);
        }
    }

    joke_remotes
}

fn bot() -> std::result::Result<(), Box<dyn std::error::Error>> {
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

    println!("init");

    // BASICS
    let mm_client = Mattermost::new(cfg.clone())?;
    let mut instance = Instance::new(mm_client.clone());
    let botdb = Arc::new(db::sqlite::new(conn));

    // TASKRUNNER
    let mut taskrunner = SequentialTaskRunner::new();
    taskrunner.add(Arc::new(Tick {}));

    // MIDDLEWARE
    let ignore_self =
        middleware::IgnoreSelf::new(mm_client.my_user_id().to_string().clone());
    if flag_debug {
        instance.add_middleware(Box::new(middleware::Debug::new("debug")));
    }
    instance.add_middleware(Box::new(ignore_self));

    // TRIGGER
    let trigger_delay_secs = Duration::from_secs(
        std::env::var("BOT_TRIGGER_DELAY_SECONDS")
            .unwrap_or("0".to_string())
            .parse()
            .unwrap(),
    );
    println!(
        "trigger configured with delay of {} seconds",
        trigger_delay_secs.as_secs()
    );
    let trigger = HandlerTrigger::new(
        botdb.clone(),
        mm_client.clone(),
        Tempo::new(),
        trigger_delay_secs,
    );
    instance.add_post_handler(Box::new(trigger));

    // EDIT
    let edits = HandlerEdit::new(botdb.clone(), mm_client.clone());
    instance.add_post_handler(Box::new(edits));

    // JOKES
    let handler_joke = joke::Handler::new(
        botdb.clone(),
        make_jokes_provider(botdb.clone()),
        mm_client.clone(),
    );
    instance.add_post_handler(Box::new(MutexedPostHandler::from(handler_joke)));

    // WEREWOLF GAME
    let ww = HandlerWW::new(mm_client.clone());
    instance.add_post_handler(Box::new(MutexedPostHandler::from(ww)));

    // SMS
    if let (Ok(login), Ok(apikey)) = (
        env::var("BOT_OCTOPUSH_LOGIN"),
        env::var("BOT_OCTOPUSH_APIKEY"),
    ) {
        let smsprov = sms::Octopush::new(&login, &apikey);
        let sms = sms::SMS::new(smsprov, botdb.clone(), mm_client.clone());
        instance.add_post_handler(Box::new(sms));
    }

    // METEO
    if let (Ok(cities), Ok(channel)) = (
        env::var("BOT_METEO_CITIES"),
        env::var("BOT_METEO_ON_CHANNEL_ID"),
    ) {
        let cities = cities.split(',').map(|p| p.to_string()).collect();
        println!(
            "exec meteo in {:?}",
            taskrunner.add(Arc::new(Meteo::new(cities, mm_client.clone(), &channel)))
        );
    }

    // RUN FOREVER
    println!("launch bot!");
    let (sender, receiver) = channel();
    let _listener_t = {
        let sender = sender.clone();
        thread::spawn(move || {
            println!("launch client thread");
            mm.listen(sender);
            println!("client thread returned");
        })
    };

    let taskrunner = Arc::new(taskrunner);
    let taskrunner_t = {
        let taskrunner = taskrunner.clone();
        thread::spawn(move || {
            println!("launch task runner");
            taskrunner.run_forever();
            println!("task runner returned");
        })
    };

    let instance_t = {
        thread::spawn(move || {
            if let Err(e) = instance.run(receiver) {
                println!("instance returned with error: {:?}", e);
            }
            println!("instance return without error");
        })
    };

    println!("wire signals");
    signal::register(Signal::SIGINT);
    signal::register(Signal::SIGTERM);

    let stop_instance_t = {
        let sender = sender.clone();
        thread::spawn(move || {
            loop {
                match signal::recv() {
                    Some(Signal::OTHER(_)) => {}
                    Some(Signal::SIGINT) | Some(Signal::SIGTERM) | None => break,
                    _ => {}
                }
            }

            while sender.send(flobot::models::Event::Shutdown).is_err() {}
            println!("graceful stop asked");
        })
    };

    println!("graceful stop: {:?}", stop_instance_t.join());
    taskrunner.stop();
    println!("taskrunner thread returned: {:?}", taskrunner_t.join());
    println!("instance thread returned: {:?}", instance_t.join());

    Ok(())
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    bot()
}
