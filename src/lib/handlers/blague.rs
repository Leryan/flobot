use super::{Handler, Result};
use crate::client::Client;
use crate::db::Blague as DBBlague;
use crate::models::GenericPost;
use rand;
use rand::Rng;
use regex::Regex;
use std::rc::Rc;

pub struct Blague<S> {
    match_del: Regex,
    store: Rc<S>,
    rng: rand::prelude::ThreadRng,
}

impl<S: DBBlague> Blague<S> {
    pub fn new(store: Rc<S>) -> Self {
        Blague {
            match_del: Regex::new(r"^!blague del (.*)")
                .expect("cannot compile blague match del regex"),
            store,
            rng: rand::thread_rng(),
        }
    }
}

impl<C: Client, S: DBBlague> Handler<C> for Blague<S> {
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

    fn handle(&mut self, data: GenericPost, client: &C) -> Result {
        let msg = data.message.as_str();

        if msg == "!blague" {
            let blagues = self.store.list(data.team_id.as_str())?;
            if blagues.len() < 1 {
                return Ok(client.send_message(data.clone(), "faut d’abord en créer")?);
            }
            let mut r = self.rng.gen_range(0, blagues.len());
            if r > blagues.len() - 1 {
                r = r - 1;
            }
            match blagues.get(r) {
                Some(blague) => return Ok(client.send_message(data, blague.text.as_str())?),
                None => return Ok(client.debug("ya une merdouille avec les blagues")?),
            };
        } else if msg == "!blague list" {
            let blagues = self.store.list(data.team_id.as_str())?;
            let mut rep = String::from("Liste des blagounettes:\n");
            for blague in blagues {
                rep.push_str(format!(" * {}: {}\n", blague.id, blague.text.as_str()).as_str());
            }

            return Ok(client.send_message(data, rep.as_str())?);
        }

        match self.match_del.captures(msg) {
            Some(captures) => {
                match captures.get(1).unwrap().as_str().trim().parse() {
                    Ok(num) => {
                        self.store.del(data.team_id.as_str(), num)?;
                        return Ok(client.send_reaction(data, "ok_hand")?);
                    }
                    Err(e) => {
                        return Ok(client.send_reply(data, format!("beurk: {:?}", e).as_str())?)
                    }
                };
            }
            None => {}
        };

        if msg.starts_with("!blague ") {
            match msg.splitn(2, " ").collect::<Vec<&str>>().get(1) {
                Some(blague) => {
                    if blague.len() > 300 {
                        return Ok(client.send_reply(
                            data.clone(),
                            "la blague est trop longue. max 300 caractères",
                        )?);
                    }
                    self.store.add(data.team_id.as_str(), blague)?;
                    return Ok(client.send_reaction(data.clone(), "ok_hand")?);
                }
                None => {
                    return Ok(client.send_reply(data.clone(), "t’as des gros doigts papa")?);
                }
            }
        }

        Ok(())
    }
}
