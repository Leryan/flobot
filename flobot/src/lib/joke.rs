use crate::db::Joke as DB;
use flobot_lib::client;
use flobot_lib::handler::Handler as BotHandler;
use flobot_lib::models::Post;
use rand::Rng;
use regex::Regex;
use reqwest::blocking::Client as RClient;
use reqwest::header as rh;
use reqwest::header::HeaderMap as rhm;
use reqwest::header::HeaderValue as rhv;
use serde::Deserialize;
use std::convert::From;
use std::sync::Arc;

#[derive(Debug)]
pub enum Error {
    Database(String),
    Client(String),
    NoData(String),
    Other(String),
}

pub type Result = std::result::Result<String, Error>;

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Client(e.to_string())
    }
}

pub trait Random {
    fn random(&self, team_id: &str) -> Result;
}

impl From<crate::db::Error> for Error {
    fn from(e: crate::db::Error) -> Self {
        match e {
            crate::db::Error::Database(e) => Error::Database(e),
            crate::db::Error::Migration(e) => Error::Database(e),
        }
    }
}

pub type Provider = Arc<dyn Random + Send + Sync>;

pub struct SelectProvider {
    remotes: Vec<Provider>,
}

impl SelectProvider {
    pub fn new(remotes: Vec<Provider>) -> Self {
        Self { remotes: remotes }
    }

    pub fn push(&mut self, remote: Provider) {
        self.remotes.push(remote)
    }

    pub fn clear(&mut self) {
        self.remotes.clear();
    }
}

impl Random for SelectProvider {
    fn random(&self, team_id: &str) -> Result {
        let l = self.remotes.len();
        let mut remote_n = rand::thread_rng().gen_range(0..l);
        for _i in 0..l {
            let res = self
                .remotes
                .get(remote_n) // [0, l) [incl, excl)
                .unwrap()
                .random(team_id);

            if res.is_ok() {
                return res;
            }

            remote_n = (remote_n + 1) % self.remotes.len();
        }

        Err(Error::NoData("no joke found :/".to_string()))
    }
}

pub struct ProviderSQLite<D> {
    db: Arc<D>,
}

impl<D> ProviderSQLite<D>
where
    D: crate::db::Joke,
{
    pub fn new(db: Arc<D>) -> Self {
        Self { db }
    }
}

impl<D> Random for ProviderSQLite<D>
where
    D: crate::db::Joke,
{
    fn random(&self, team_id: &str) -> Result {
        let l = self.db.count(team_id)?;
        if l < 1 {
            return Err(Error::NoData("no joke in db".to_string()));
        }
        let joke = self.db.pick(team_id, rand::thread_rng().gen_range(0..l))?;
        match joke {
            Some(b) => Ok(b.text),
            None => Err(Error::NoData("cannot find that joke".to_string())),
        }
    }
}

pub struct ProviderBadJokes {
    c: RClient,
    match_: Regex,
}

impl ProviderBadJokes {
    pub fn new() -> Self {
        let mut hm = rhm::new();
        hm.insert(
            rh::REFERER,
            rhv::from_str("https://random-ize.com/bad-jokes/").unwrap(),
        );
        hm.insert(rh::ACCEPT, rhv::from_str("text/html, */*; q=0.01").unwrap());
        hm.insert(
            rh::ACCEPT_LANGUAGE,
            rhv::from_str("en-US,en;q=0.5").unwrap(),
        );
        hm.insert(rh::TE, rhv::from_str("trailers").unwrap());
        hm.insert(rh::CACHE_CONTROL, rhv::from_str("no-cache").unwrap());
        hm.insert(rh::PRAGMA, rhv::from_str("no-cache").unwrap());
        hm.insert("Sec-Fetch-Site", rhv::from_str("same-origin").unwrap());
        hm.insert("Sec-Fetch-Mode", rhv::from_str("cors").unwrap());
        hm.insert("Sec-Fetch-Dest", rhv::from_str("empty").unwrap());
        hm.insert("X-Requested-With", rhv::from_str("XMLHttpRequest").unwrap());
        let c = RClient::builder()
            .user_agent(
                "Mozilla/5.0 (X11; Linux x86_64; rv:92.0) Gecko/20100101 Firefox/92.0",
            )
            .default_headers(hm)
            .referer(true)
            .timeout(std::time::Duration::from_secs(2))
            .gzip(true)
            .build()
            .unwrap();
        Self {
            c: c,
            match_: Regex::new(".*<font[^>]*>(.*)<br><br>(.*)</font>.*").unwrap(),
        }
    }
}

impl Random for ProviderBadJokes {
    fn random(&self, _team_id: &str) -> Result {
        let q = self
            .c
            .get("https://random-ize.com/bad-jokes/bad-jokes-f.php");

        let s = q.send()?.text_with_charset("utf-8")?;
        match self.match_.captures(&s) {
            Some(captures) => {
                return Ok(format!(
                    "{}\n…\n…\n{}",
                    captures.get(1).unwrap().as_str(),
                    captures.get(2).unwrap().as_str()
                ));
            }
            None => {
                return Err(Error::NoData(
                    "no match for random-ize.com/bad-jokes/".to_string(),
                ))
            }
        };
    }
}

pub struct ProviderBlaguesAPI {
    client: RClient,
}

#[derive(Deserialize)]
struct BlaguesAPIResponse {
    pub joke: String,
    pub answer: String,
}

impl ProviderBlaguesAPI {
    pub fn new(token: &str) -> Self {
        let mut hm = rhm::new();
        hm.insert(
            rh::AUTHORIZATION,
            rhv::from_str(&format!("Bearer {}", token)).unwrap(),
        );
        Self {
            client: RClient::builder().default_headers(hm).build().unwrap(),
        }
    }
}

impl Random for ProviderBlaguesAPI {
    fn random(&self, _team_id: &str) -> Result {
        let joke: BlaguesAPIResponse = self
            .client
            .get("https://www.blagues-api.fr/api/random")
            .send()?
            .json()?;
        return Ok(format!("{}\n…\n…\n{}", joke.joke, joke.answer));
    }
}

pub struct ProviderFile {
    pub urls: Vec<String>,
}

impl Random for ProviderFile {
    fn random(&self, _team_id: &str) -> Result {
        let rnd = rand::random::<usize>() % self.urls.len();
        Ok(self.urls[rnd].clone())
    }
}

impl From<Error> for flobot_lib::handler::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::Client(s) => flobot_lib::handler::Error::Timeout(s),
            Error::NoData(s) => flobot_lib::handler::Error::Database(s),
            Error::Other(s) => flobot_lib::handler::Error::Other(s),
            Error::Database(s) => flobot_lib::handler::Error::Database(s),
        }
    }
}

pub struct Handler<R, S, C> {
    match_del: Regex,
    store: Arc<S>,
    remotes: R,
    client: C,
}

impl<R, S, C> Handler<R, S, C> {
    pub fn new(store: Arc<S>, remotes: R, client: C) -> Self {
        Handler {
            match_del: Regex::new(r"^!joke del (.*)")
                .expect("cannot compile joke match del regex"),
            store,
            remotes,
            client,
        }
    }
}

impl<R, C, S> BotHandler for Handler<R, S, C>
where
    C: client::Sender,
    S: DB,
    R: Random,
{
    type Data = Post;

    fn name(&self) -> String {
        "joke".into()
    }
    fn help(&self) -> Option<String> {
        Some(
            "```
!joke # quick, a joke, now!
!joke <register a joke>
!joke list
!joke del <num>
```"
            .to_string(),
        )
    }

    fn handle(&self, post: &Post) -> flobot_lib::handler::Result {
        let msg = &post.message;

        if msg == "!joke" {
            let joke = self.remotes.random(&post.team_id)?;
            return Ok(self.client.post(&post.nmessage(&joke))?);
        } else if msg == "!joke list" {
            let jokes = self.store.list(&post.team_id)?;
            let mut rep = String::from("Available jokes:\n");
            for joke in jokes {
                rep.push_str(&format!(" * {}: {}\n", joke.id, &joke.text));
            }

            return Ok(self.client.post(&post.nmessage(&rep))?);
        }

        match self.match_del.captures(msg) {
            Some(captures) => {
                match captures.get(1).unwrap().as_str().trim().parse() {
                    Ok(num) => {
                        self.store.del(&post.team_id, num)?;
                        return Ok(self.client.reaction(post, "ok_hand")?);
                    }
                    Err(e) => {
                        return Ok(self
                            .client
                            .reply(post, &format!("beurk: {:?}", e))?)
                    }
                };
            }
            None => {}
        };

        if msg.starts_with("!joke ") {
            match msg.splitn(2, " ").collect::<Vec<&str>>().get(1) {
                Some(joke) => {
                    if joke.len() > 300 {
                        return Ok(self
                            .client
                            .reply(post, "too long: max 300 chars")?);
                    }
                    self.store.add(&post.team_id, joke)?;
                    return Ok(self.client.reaction(post, "ok_hand")?);
                }
                None => {
                    return Ok(self.client.reply(post, "nope")?);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_bad_jokes() -> std::result::Result<(), Error> {
        let bj = ProviderBadJokes::new();
        let _ = bj.random("tid1")?;
        let _ = bj.random("tid2")?;
        Ok(())
    }

    #[test]
    fn test_blaguesapi_random() -> std::result::Result<(), Error> {
        use dotenv;
        use std::env;
        dotenv::from_filename("flobot.env").ok();

        let test_token = env::var("BOT_BLAGUESAPI_TOKEN").unwrap();
        let ba = ProviderBlaguesAPI::new(&test_token);
        let r1 = ba.random("tid")?;
        assert_ne!("", &r1);
        let r2 = ba.random("tid")?;
        assert_ne!("", &r2);
        assert_ne!(r1, r2);
        Ok(())
    }
}
