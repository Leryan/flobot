use chrono::Timelike; // chrono Duration::with_hour etc…
use flobot_lib::client::Sender;
use flobot_lib::models::Post;
use flobot_lib::task::{cduration_from_secs, Error, ExecIn, Now, Task};
use reqwest::blocking::Client;
use std::time::Duration;

pub struct Meteo<S: Sender> {
    client: S,
    on_channel_id: String,
    cities: Vec<String>,
}

impl<S: Sender> Meteo<S> {
    pub fn new(cities: Vec<String>, client: S, on_channel_id: &str) -> Self {
        Self {
            on_channel_id: on_channel_id.to_string(),
            client: client,
            cities: cities,
        }
    }
}

impl<S: Sender> Task for Meteo<S> {
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
            let r = Client::new()
                .get(&url)
                .query(&[("format", "%l: %c %t")])
                .send();

            if let Err(e) = r {
                return Err(Error::CannotExec((
                    Duration::from_secs(24 * 3600),
                    e.to_string(),
                )));
            }

            let v = r.unwrap();
            if v.status().is_client_error() {
                return Err(Error::CannotExec((
                    Duration::from_secs(24 * 3600),
                    v.status().to_string(),
                )));
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

        let tomorrow = now.with_hour(7).unwrap().with_minute(23).unwrap()
            + cduration_from_secs(24 * 3600);
        Ok((tomorrow - now).to_std().unwrap())
    }
}
