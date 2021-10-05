#[macro_use]
extern crate diesel_migrations;
use dotenv;
use flobot::db;
use flobot::joke;
use flobot::weather::Meteo;
use flobot::{
    edits::Edit as HandlerEdit, pinterest::Pinterest, sms,
    trigger::Trigger as HandlerTrigger, werewolf::Handler as HandlerWW,
};
use flobot_lib::client::Getter;
use flobot_lib::conf::Conf;
use flobot_lib::handler::MutexedHandler;
use flobot_lib::instance::Instance;
use flobot_lib::middleware;
use flobot_lib::models::Event;
use flobot_lib::task::*;
use flobot_lib::tempo::Tempo;
use flobot_mattermost::client::Mattermost;
use signal_libc::signal::{self, Signal};
use simple_server as ss;
use std::env;
use std::fs;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

embed_migrations!();

fn make_jokes_provider(botdb: Arc<db::sqlite::Sqlite>) -> joke::SelectProvider {
    let mut joke_remotes = joke::SelectProvider::new(vec![]);
    joke_remotes.push(Arc::new(joke::ProviderBadJokes::new()));
    joke_remotes.push(Arc::new(joke::ProviderSQLite::new(botdb)));
    if let Ok(token) = env::var("BOT_BLAGUESAPI_TOKEN") {
        let blaguesapi = joke::ProviderBlaguesAPI::new(&token);
        joke_remotes.push(Arc::new(blaguesapi));
    }

    if let Ok(filepath) = env::var("BOT_BLAGUES_URLS") {
        if let Ok(content) = fs::read_to_string(filepath.clone()) {
            let mut urls = vec![];
            for line in content.split("\n") {
                urls.push(line.to_string());
            }

            joke_remotes.push(Arc::new(joke::ProviderFile { urls }));
        } else {
            println!("cannot read jokes from {}", filepath);
        }
    }

    joke_remotes
}

fn bot() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("Launch version {}", flobot_lib::BUILD_GIT_HASH);
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
    let mut jokeprovider = make_jokes_provider(botdb.clone());

    // PINTEREST
    let mut handler: Option<_> = None;
    if let (Ok(client_id), Ok(client_secret), Ok(board_id), Ok(redirect)) = (
        env::var("PINTEREST_CLIENT_ID"),
        env::var("PINTEREST_CLIENT_SECRET"),
        env::var("PINTEREST_BOARD_ID"),
        env::var("PINTEREST_REDIRECT"),
    ) {
        println!("loading pinterest");
        let pinterest = Arc::new(Pinterest::new(
            &client_id,
            &client_secret,
            &redirect,
            &board_id,
            mm_client.clone(),
        ));

        jokeprovider.push(pinterest.clone());
        taskrunner.add(pinterest.clone());

        handler = Some(
            move |request: ss::Request<Vec<u8>>,
                  mut response: ss::ResponseBuilder|
                  -> ss::ResponseResult {
                let furl = format!("http://localhost{}", request.uri());

                let mut code = "".to_string();
                let mut state = "".to_string();

                if let Ok(url) = url::Url::parse(&furl) {
                    for qp in url.query_pairs() {
                        if qp.0 == "code" {
                            code = qp.1.to_string();
                        } else if qp.0 == "state" {
                            state = qp.1.to_string();
                        }
                    }
                }

                println!("pinterest: got code {}", code);
                if pinterest.authenticate(&code, &state) {
                    println!("authenticated!");
                    return Ok(response
                        .status(200)
                        .body("Authenticated!".as_bytes().to_vec())?);
                }

                println!("pinterest: failed to authenticate");

                Ok(response
                    .status(500)
                    .body("NOT AUTHENTICATED".as_bytes().to_vec())?)
            },
        );
    }

    let handler_joke =
        joke::Handler::new(botdb.clone(), jokeprovider, mm_client.clone());
    instance.add_post_handler(Box::new(MutexedHandler::from(handler_joke)));

    // WEREWOLF GAME
    let ww = HandlerWW::new(mm_client.clone());
    instance.add_post_handler(Box::new(MutexedHandler::from(ww)));

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

    if let Some(handler) = handler {
        println!("starting webserver with declared handler");
        let _www_t = thread::spawn(move || loop {
            // survive crashes from webserver
            let hh = handler.clone();
            let r = thread::spawn(move || {
                let mut server = ss::Server::new(hh);
                server.dont_serve_static_files();
                println!("launch webserver on localhost:6799");
                server.listen("localhost", "6799");
            })
            .join();
            println!("webserver thread loop returned: {:?}", r);
        });
    }

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

            while sender.send(Event::Shutdown).is_err() {}
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
