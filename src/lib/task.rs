use crate::db::tempo::Tempo;
use crate::models::Post;
use chrono::{self, DateTime, Duration as CDuration, Local, Timelike};
use reqwest::blocking::Client;
use std::convert::From;
use std::sync::Arc;
use std::sync::Mutex;
use std::{thread, time, time::Duration};

pub fn cduration_from_secs(secs: u64) -> CDuration {
    CDuration::from_std(Duration::from_secs(secs)).unwrap()
}

pub type Now = DateTime<Local>;

#[derive(Debug, Clone)]
pub enum Error {
    /// The task runner should reschedule the task as soon as possible.
    Reschedule(String),
    /// The task runner should skip this task and reschedule as task says.
    CannotExec((ExecIn, String)),
    /// The task should be rescheduled in time as a function of exponential.
    ExpRetry(String),
}

/// Provide a basic error management that will reschedule the call.
impl From<reqwest::Error> for Error {
    fn from(re: reqwest::Error) -> Self {
        Error::Reschedule(re.to_string())
    }
}

pub type ExecIn = time::Duration;

pub type RunnableTask = Arc<dyn Task + Send + Sync>;

/// Task implements work to be done regularly.
/// The &Duration passed to the task
pub trait Task {
    fn name(&self) -> String;
    fn init_exec(&self, now: Now) -> ExecIn;
    fn exec(&self, now: Now) -> Result<ExecIn, Error>;
}

pub struct SequentialTaskRunner {
    tasks: Vec<RunnableTask>,
    tempo: Tempo, // contain task names
    cont: Mutex<bool>,
}
/// TaskRunner will optimistically run tasks, sequentially. No threading used.
/// Pauses for 10 seconds between each run loop.
///
/// You should run this in a thread and avoid memory sharing, especially with
/// tasks that run for more than a few seconds.
///
/// Stacking time consuming tasks will simply delay all tasks.
///
/// The minimum ExecAgainIn time for a task will always be 60 seconds.
impl SequentialTaskRunner {
    pub fn new() -> Self {
        Self {
            tasks: vec![],
            tempo: Tempo::new(),
            cont: Mutex::new(true), // TODO: use Arc<Mutex<bool>>?
        }
    }

    pub fn add(&mut self, task: RunnableTask) -> Duration {
        let exec_in = task.init_exec(Local::now()).max(Duration::from_secs(3));
        self.tempo.set(task.name(), exec_in);
        self.tasks.push(task);
        exec_in
    }

    pub fn run_forever(&self) {
        while *self.cont.lock().unwrap() {
            for task in self.tasks.iter() {
                let key = task.name();
                if self.tempo.exists(&key) {
                    continue; // skip task and run only when key is removed at future access.
                }
                match task.exec(Local::now()) {
                    Err(e) => {
                        println!("task {} failed: {:?}", key, e);
                        match e {
                            Error::Reschedule(_) => self.tempo.set(key, Duration::from_secs(123)),
                            Error::CannotExec((exec_in, _)) => self.tempo.set(key, exec_in),
                            Error::ExpRetry(_) => self.tempo.set(key, Duration::from_secs(196)), // TODO: implement exp
                        };
                    }
                    Ok(rai) => {
                        let dur = rai.max(Duration::from_secs(60));
                        let at = Local::now() + CDuration::from_std(dur).unwrap();
                        println!("task {} next execution scheduled at {}", task.name(), at);
                        self.tempo.set(key, dur);
                    }
                };
            }

            thread::sleep(Duration::from_secs(1));
        }
    }

    pub fn stop(&self) {
        *self.cont.lock().unwrap() = false
    }
}

pub struct Tick {}

impl Task for Tick {
    fn name(&self) -> String {
        "tick".into()
    }

    fn exec(&self, _now: Now) -> Result<ExecIn, Error> {
        println!("tick…");
        Ok(Duration::from_secs(1))
    }

    fn init_exec(&self, _now: Now) -> ExecIn {
        Duration::from_secs(0)
    }
}

pub struct Meteo<S: crate::client::Sender> {
    client: S,
    on_channel_id: String,
    cities: Vec<String>,
}

impl<S: crate::client::Sender> Meteo<S> {
    pub fn new(cities: Vec<String>, client: S, on_channel_id: &str) -> Self {
        Self {
            on_channel_id: on_channel_id.to_string(),
            client: client,
            cities: cities,
        }
    }
}

impl<S: crate::client::Sender> Task for Meteo<S> {
    fn name(&self) -> String {
        "meteo".into()
    }

    fn init_exec(&self, now: Now) -> ExecIn {
        let mut sched = now.with_hour(7).unwrap().with_minute(23).unwrap();
        if sched < now {
            sched = sched + cduration_from_secs(24 * 3600);
        }
        (sched - now).to_std().unwrap()
    }

    fn exec(&self, now: Now) -> Result<ExecIn, Error> {
        let mut msg = String::from("Mééééééééééééétéoooooooooooo :\n");

        for city in self.cities.iter() {
            let url = format!("https://wttr.in/{}", city);
            let r = Client::new().get(&url).query(&[("format", "%l: %c %t")]).send();

            if let Err(e) = r {
                return Err(Error::CannotExec((Duration::from_secs(24 * 3600), e.to_string())));
            }

            let v = r.unwrap();
            if v.status().is_client_error() {
                return Err(Error::CannotExec((Duration::from_secs(24 * 3600), v.status().to_string())));
            }
            if v.status().is_server_error() {
                return Err(Error::ExpRetry(v.status().to_string()));
            }

            msg.push_str(&format!(" * {}\n", &v.text().unwrap()));
        }

        let post = Post::with_message(&msg).nchannel(&self.on_channel_id);
        if self.client.post(&post).is_err() {
            return Err(Error::ExpRetry("cannot post".into()));
        }

        let tomorrow = now.with_hour(7).unwrap().with_minute(23).unwrap() + cduration_from_secs(24 * 3600);
        Ok((tomorrow - now).to_std().unwrap())
    }
}
