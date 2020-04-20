use super::Blague;
use super::Error;
use super::Result;
use regex::Regex;
use std::convert::From;
use std::rc::Rc;
use std::str::from_utf8;

impl From<crate::db::Error> for super::Error {
    fn from(e: crate::db::Error) -> Self {
        match e {
            crate::db::Error::Database(e) => Error::Database(e),
            crate::db::Error::Migration(e) => Error::Database(e),
        }
    }
}

pub struct Select<R>
where
    R: rand::Rng,
{
    remotes: Vec<Box<dyn Blague>>,
    rng: R,
}

impl<R: rand::Rng> Select<R> {
    pub fn new(rng: R, first: Box<dyn Blague>, second: Box<dyn Blague>) -> Self {
        Self {
            remotes: vec![first, second],
            rng,
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
    fn random(&mut self, team_id: &str) -> Result {
        let l = self.remotes.len();
        self.remotes
            .get_mut(self.rng.gen_range(0, l)) // [0, l) [incl, excl)
            .unwrap()
            .random(team_id)
    }
}

pub struct Sqlite<R, D>
where
    R: rand::Rng,
{
    db: Rc<D>,
    rng: R,
}

impl<R, D> Sqlite<R, D>
where
    R: rand::Rng,
    D: crate::db::Blague,
{
    pub fn new(rng: R, db: Rc<D>) -> Self {
        Self { db, rng }
    }
}

impl<R, D> Blague for Sqlite<R, D>
where
    R: rand::Rng,
    D: crate::db::Blague,
{
    fn random(&mut self, team_id: &str) -> Result {
        let l = self.db.count(team_id)?;
        let blague = self.db.pick(team_id, self.rng.gen_range(0, l))?;
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
        Self {
            c: reqwest::blocking::Client::new(),
            match_: Regex::new(".*<font.*>(.*)<br><br>(.*)</font>.*").unwrap(),
        }
    }
}

impl Blague for BadJokes {
    fn random(&mut self, _team_id: &str) -> Result {
        let r = self
            .c
            .get("https://random-ize.com/bad-jokes/bad-jokes-f.php")
            .header("Referer", "https://random-ize.com/bad-jokes/")
            .timeout(std::time::Duration::from_secs(2))
            .send()?
            .bytes()?;
        let s = from_utf8(r.as_ref()).unwrap();
        match self.match_.captures(s) {
            Some(captures) => {
                return Ok(format!(
                    "{}\n\n{}",
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
