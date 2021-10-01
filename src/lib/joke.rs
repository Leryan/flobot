use crate::client;
use crate::db::Joke as DB;
use crate::handlers::Handler as BotHandler;
use crate::models::Post;
use regex::Regex;
use reqwest::blocking::Client as RClient;
use reqwest::header as rh;
use reqwest::header::HeaderMap as rhm;
use reqwest::header::HeaderValue as rhv;
use serde::Deserialize;
use std::cell::RefCell;
use std::convert::From;
use std::rc::Rc;
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
        if e.is_builder() || e.is_status() || e.is_timeout() {
            return Self::Client(e.to_string());
        }

        Self::Other(e.to_string())
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

pub type Provider = Arc<dyn Random>;

pub struct SelectProvider<R> {
    remotes: Vec<Provider>,
    rng: RefCell<R>,
}

impl<R: rand::Rng> SelectProvider<R> {
    pub fn new(rng: R, remotes: Vec<Provider>) -> Self {
        Self {
            remotes: remotes,
            rng: RefCell::new(rng),
        }
    }

    pub fn push(&mut self, remote: Provider) {
        self.remotes.push(remote)
    }
}

impl<R> Random for SelectProvider<R>
where
    R: rand::Rng,
{
    fn random(&self, team_id: &str) -> Result {
        let l = self.remotes.len();
        let mut remote_n = self.rng.borrow_mut().gen_range(0..l);
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

pub struct ProviderSQLite<R, D>
where
    R: rand::Rng,
{
    db: Rc<D>,
    rng: RefCell<R>,
}

impl<R, D> ProviderSQLite<R, D>
where
    R: rand::Rng,
    D: crate::db::Joke,
{
    pub fn new(rng: R, db: Rc<D>) -> Self {
        Self {
            db,
            rng: RefCell::new(rng),
        }
    }
}

impl<R, D> Random for ProviderSQLite<R, D>
where
    R: rand::Rng,
    D: crate::db::Joke,
{
    fn random(&self, team_id: &str) -> Result {
        let l = self.db.count(team_id)?;
        if l < 1 {
            return Err(Error::NoData("no joke in db".to_string()));
        }
        let blague = self.db.pick(team_id, self.rng.borrow_mut().gen_range(0..l))?;
        match blague {
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
        hm.insert(rh::REFERER, rhv::from_str("https://random-ize.com/bad-jokes/").unwrap());
        hm.insert(rh::ACCEPT, rhv::from_str("text/html, */*; q=0.01").unwrap());
        hm.insert(rh::ACCEPT_LANGUAGE, rhv::from_str("en-US,en;q=0.5").unwrap());
        hm.insert(rh::TE, rhv::from_str("trailers").unwrap());
        hm.insert(rh::CACHE_CONTROL, rhv::from_str("no-cache").unwrap());
        hm.insert(rh::PRAGMA, rhv::from_str("no-cache").unwrap());
        hm.insert("Sec-Fetch-Site", rhv::from_str("same-origin").unwrap());
        hm.insert("Sec-Fetch-Mode", rhv::from_str("cors").unwrap());
        hm.insert("Sec-Fetch-Dest", rhv::from_str("empty").unwrap());
        hm.insert("X-Requested-With", rhv::from_str("XMLHttpRequest").unwrap());
        let c = RClient::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:92.0) Gecko/20100101 Firefox/92.0")
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
        let q = self.c.get("https://random-ize.com/bad-jokes/bad-jokes-f.php");

        let s = q.send()?.text_with_charset("utf-8")?;
        match self.match_.captures(&s) {
            Some(captures) => {
                return Ok(format!(
                    "{}\n…\n…\n{}",
                    captures.get(1).unwrap().as_str(),
                    captures.get(2).unwrap().as_str()
                ));
            }
            None => return Err(Error::NoData("no match for random-ize.com/bad-jokes/".to_string())),
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
        hm.insert(rh::AUTHORIZATION, rhv::from_str(&format!("Bearer {}", token)).unwrap());
        Self {
            client: RClient::builder().default_headers(hm).build().unwrap(),
        }
    }
}

impl Random for ProviderBlaguesAPI {
    fn random(&self, _team_id: &str) -> Result {
        let joke: BlaguesAPIResponse = self.client.get("https://www.blagues-api.fr/api/random").send()?.json()?;
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

impl From<Error> for crate::handlers::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::Client(s) => crate::handlers::Error::Timeout(s),
            Error::NoData(s) => crate::handlers::Error::Database(s),
            Error::Other(s) => crate::handlers::Error::Other(s),
            Error::Database(s) => crate::handlers::Error::Database(s),
        }
    }
}

pub struct Handler<R, S, C> {
    match_del: Regex,
    store: Rc<S>,
    remotes: R,
    client: C,
}

impl<R, S, C> Handler<R, S, C> {
    pub fn new(store: Rc<S>, remotes: R, client: C) -> Self {
        Handler {
            match_del: Regex::new(r"^!blague del (.*)").expect("cannot compile blague match del regex"),
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

    fn name(&self) -> &str {
        "blague"
    }
    fn help(&self) -> Option<String> {
        Some(
            "```
!blague # raconte une blague
!blague <une blague> # enregistre une nouvelle blague
!blague list
!blague del <num>
```"
            .to_string(),
        )
    }

    fn handle(&self, post: &Post) -> crate::handlers::Result {
        let msg = post.message.as_str();

        if msg == "!blague" {
            println!("asked a joke");
            let blague = self.remotes.random(&post.team_id)?;
            return Ok(self.client.message(post, &blague)?);
        } else if msg == "!blague list" {
            let blagues = self.store.list(&post.team_id)?;
            let mut rep = String::from("Liste des blagounettes enregistrées à la meuson:\n");
            for blague in blagues {
                rep.push_str(&format!(" * {}: {}\n", blague.id, &blague.text));
            }

            return Ok(self.client.message(post, &rep)?);
        }

        match self.match_del.captures(msg) {
            Some(captures) => {
                match captures.get(1).unwrap().as_str().trim().parse() {
                    Ok(num) => {
                        self.store.del(&post.team_id, num)?;
                        return Ok(self.client.reaction(post, "ok_hand")?);
                    }
                    Err(e) => return Ok(self.client.reply(post, &format!("beurk: {:?}", e))?),
                };
            }
            None => {}
        };

        if msg.starts_with("!blague ") {
            match msg.splitn(2, " ").collect::<Vec<&str>>().get(1) {
                Some(blague) => {
                    if blague.len() > 300 {
                        return Ok(self.client.reply(post, "la blague est trop longue. max 300 caractères")?);
                    }
                    self.store.add(&post.team_id, blague)?;
                    return Ok(self.client.reaction(post, "ok_hand")?);
                }
                None => {
                    return Ok(self.client.reply(post, "t’as des gros doigts papa")?);
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
        assert_ne!("", r1.as_str());
        let r2 = ba.random("tid")?;
        assert_ne!("", r2.as_str());
        assert_ne!(r1, r2);
        Ok(())
    }
}
