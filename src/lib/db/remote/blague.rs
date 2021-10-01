use super::Blague;
use super::Error;
use super::Result;
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

impl From<crate::db::Error> for super::Error {
    fn from(e: crate::db::Error) -> Self {
        match e {
            crate::db::Error::Database(e) => Error::Database(e),
            crate::db::Error::Migration(e) => Error::Database(e),
        }
    }
}

pub type Remote = Arc<dyn Blague>;

pub struct Select<R> {
    remotes: Vec<Remote>,
    rng: RefCell<R>,
}

impl<R: rand::Rng> Select<R> {
    pub fn new(rng: R, remotes: Vec<Remote>) -> Self {
        Self {
            remotes: remotes,
            rng: RefCell::new(rng),
        }
    }

    pub fn push(&mut self, remote: Remote) {
        self.remotes.push(remote)
    }
}

impl<R> Blague for Select<R>
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

pub struct Sqlite<R, D>
where
    R: rand::Rng,
{
    db: Rc<D>,
    rng: RefCell<R>,
}

impl<R, D> Sqlite<R, D>
where
    R: rand::Rng,
    D: crate::db::Blague,
{
    pub fn new(rng: R, db: Rc<D>) -> Self {
        Self {
            db,
            rng: RefCell::new(rng),
        }
    }
}

impl<R, D> Blague for Sqlite<R, D>
where
    R: rand::Rng,
    D: crate::db::Blague,
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

pub struct BadJokes {
    c: RClient,
    match_: Regex,
}

impl BadJokes {
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

impl Blague for BadJokes {
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

pub struct BlaguesAPI {
    client: RClient,
}

#[derive(Deserialize)]
struct BlaguesAPIResponse {
    pub joke: String,
    pub answer: String,
}

impl BlaguesAPI {
    pub fn new(token: &str) -> Self {
        let mut hm = rhm::new();
        hm.insert(rh::AUTHORIZATION, rhv::from_str(&format!("Bearer {}", token)).unwrap());
        Self {
            client: RClient::builder().default_headers(hm).build().unwrap(),
        }
    }
}

impl Blague for BlaguesAPI {
    fn random(&self, _team_id: &str) -> Result {
        let joke: BlaguesAPIResponse = self.client.get("https://www.blagues-api.fr/api/random").send()?.json()?;
        return Ok(format!("{}\n…\n…\n{}", joke.joke, joke.answer));
    }
}

pub struct URLs {
    pub urls: Vec<String>,
}

impl Blague for URLs {
    fn random(&self, _team_id: &str) -> Result {
        let rnd = rand::random::<usize>() % self.urls.len();
        Ok(self.urls[rnd].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_bad_jokes() -> std::result::Result<(), Error> {
        let bj = BadJokes::new();
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
        let ba = BlaguesAPI::new(&test_token);
        let r1 = ba.random("tid")?;
        assert_ne!("", r1.as_str());
        let r2 = ba.random("tid")?;
        assert_ne!("", r2.as_str());
        assert_ne!(r1, r2);
        Ok(())
    }
}
