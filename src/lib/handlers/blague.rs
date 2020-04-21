use super::{Handler, Result};
use crate::client;
use crate::db::remote;
use crate::db::Blague as DBBlague;
use crate::models::GenericPost;
use regex::Regex;
use std::convert::From;
use std::rc::Rc;

impl From<remote::Error> for crate::handlers::Error {
    fn from(e: remote::Error) -> Self {
        match e {
            remote::Error::Client(s) => crate::handlers::Error::Timeout(s),
            remote::Error::NoData(s) => crate::handlers::Error::Database(s),
            remote::Error::Other(s) => crate::handlers::Error::Other(s),
            remote::Error::Database(s) => crate::handlers::Error::Database(s),
        }
    }
}

pub struct Blague<R, S, C> {
    match_del: Regex,
    store: Rc<S>,
    remote: R,
    client: Rc<C>,
}

impl<R, S, C> Blague<R, S, C> {
    pub fn new(store: Rc<S>, remote: R, client: Rc<C>) -> Self {
        Blague {
            match_del: Regex::new(r"^!blague del (.*)")
                .expect("cannot compile blague match del regex"),
            store,
            remote,
            client,
        }
    }
}

impl<R, C, S> Handler for Blague<R, S, C>
where
    C: client::Sender,
    S: DBBlague,
    R: remote::Blague,
{
    type Data = GenericPost;

    fn name(&self) -> &str {
        "blague"
    }
    fn help(&self) -> Option<&str> {
        Some(
            "```
!blague # raconte une blague
!blague <une blague> # enregistre une nouvelle blague
!blague list
!blague del <num>
```",
        )
    }

    fn handle(&mut self, data: GenericPost) -> Result {
        let msg: &str = &data.message;

        if msg == "!blague" {
            let blague = self.remote.random(&data.team_id)?;
            return Ok(self.client.message(data, &blague)?);
        } else if msg == "!blague list" {
            let blagues = self.store.list(&data.team_id)?;
            let mut rep = String::from("Liste des blagounettes enregistrées à la meuson:\n");
            for blague in blagues {
                rep.push_str(&format!(" * {}: {}\n", blague.id, &blague.text));
            }

            return Ok(self.client.message(data, &rep)?);
        }

        match self.match_del.captures(msg) {
            Some(captures) => {
                match captures.get(1).unwrap().as_str().trim().parse() {
                    Ok(num) => {
                        self.store.del(&data.team_id, num)?;
                        return Ok(self.client.reaction(data, "ok_hand")?);
                    }
                    Err(e) => return Ok(self.client.reply(data, &format!("beurk: {:?}", e))?),
                };
            }
            None => {}
        };

        if msg.starts_with("!blague ") {
            match msg.splitn(2, " ").collect::<Vec<&str>>().get(1) {
                Some(blague) => {
                    if blague.len() > 300 {
                        return Ok(self
                            .client
                            .reply(data, "la blague est trop longue. max 300 caractères")?);
                    }
                    self.store.add(&data.team_id, blague)?;
                    return Ok(self.client.reaction(data, "ok_hand")?);
                }
                None => {
                    return Ok(self.client.reply(data, "t’as des gros doigts papa")?);
                }
            }
        }

        Ok(())
    }
}
