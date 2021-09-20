use super::Blague;
use super::Error;
use super::Result;
use regex::Regex;
use serde::Deserialize;
use std::cell::RefCell;
use std::convert::From;
use std::rc::Rc;

impl From<crate::db::Error> for super::Error {
    fn from(e: crate::db::Error) -> Self {
        match e {
            crate::db::Error::Database(e) => Error::Database(e),
            crate::db::Error::Migration(e) => Error::Database(e),
        }
    }
}

pub struct Select<R> {
    remotes: Vec<Box<dyn Blague>>,
    rng: RefCell<R>,
}

impl<R: rand::Rng> Select<R> {
    pub fn new(rng: R, remotes: Vec<Box<dyn Blague>>) -> Self {
        Self {
            remotes: remotes,
            rng: RefCell::new(rng),
        }
    }

    pub fn push(&mut self, remote: Box<dyn Blague>) {
        self.remotes.push(remote)
    }
}

impl<R> Blague for Select<R>
where
    R: rand::Rng,
{
    fn random(&self, team_id: &str) -> Result {
        let l = self.remotes.len();
        self.remotes
            .get(self.rng.borrow_mut().gen_range(0..l)) // [0, l) [incl, excl)
            .unwrap()
            .random(team_id)
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
        let blague = self.db.pick(team_id, self.rng.borrow_mut().gen_range(0..l))?;
        match blague {
            Some(b) => Ok(b.text),
            None => Err(Error::NoData("cannot find that joke".to_string())),
        }
    }
}

pub struct BadJokes {
    c: reqwest::blocking::Client,
    match_: Regex,
}

impl BadJokes {
    pub fn new() -> Self {
        use reqwest::header as h;
        use reqwest::header::HeaderValue as hv;
        let mut hm = reqwest::header::HeaderMap::new();
        hm.insert(h::REFERER, hv::from_str("https://random-ize.com/bad-jokes/").unwrap());
        hm.insert(h::ACCEPT, hv::from_str("text/html, */*; q=0.01").unwrap());
        hm.insert(h::ACCEPT_LANGUAGE, hv::from_str("en-US,en;q=0.5").unwrap());
        hm.insert("x-requested-with", hv::from_str("XMLHttpRequest").unwrap());
        let c = reqwest::blocking::Client::builder()
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
    client: reqwest::blocking::Client,
}

#[derive(Deserialize)]
struct BlaguesAPIResponse {
    pub id: u64,
    #[serde(rename = "type")]
    pub type_: String,
    pub joke: String,
    pub answer: String,
}

impl BlaguesAPI {
    pub fn new(token: &str) -> Self {
        let mut hm = reqwest::header::HeaderMap::new();
        hm.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        );
        Self {
            client: reqwest::blocking::Client::builder().default_headers(hm).build().unwrap(),
        }
    }
}

impl Blague for BlaguesAPI {
    fn random(&self, _team_id: &str) -> Result {
        let joke: BlaguesAPIResponse = self.client.get("https://www.blagues-api.fr/api/random").send()?.json()?;
        return Ok(format!("{}\n…\n…\n{}", joke.joke, joke.answer));
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

        let test_token = env::var("BOT_BLAGUESAPI_TOKEN");
        if let Err(_) = test_token.clone() {
            return Ok(());
        }

        let ba = BlaguesAPI::new(&test_token.unwrap());
        let _ = ba.random("tid")?;
        let _ = ba.random("tid")?;
        Ok(())
    }
}
