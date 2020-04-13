use super::{Error, Handler, Result};
use crate::client::Client;
use crate::db::Blague as DBBlague;
use crate::models::Blague as MBlague;
use crate::models::GenericPost;
use rand;
use rand::Rng;
use regex::Regex;
use std::rc::Rc;
use std::str::from_utf8;

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

    fn rand_blague(&self) -> std::result::Result<Option<String>, Error> {
        let c = reqwest::blocking::Client::new();
        let r = c
            .get("https://random-ize.com/bad-jokes/bad-jokes-f.php")
            .header("Referer", "https://random-ize.com/bad-jokes/")
            .timeout(std::time::Duration::from_secs(2))
            .send()?
            .bytes()?;
        let s = from_utf8(r.as_ref()).unwrap();
        match Regex::new(".*<font.*>(.*)<br><br>(.*)</font>.*")
            .unwrap()
            .captures(s)
        {
            Some(captures) => {
                return Ok(Some(format!(
                    "{}\n\n{}",
                    captures.get(1).unwrap().as_str(),
                    captures.get(2).unwrap().as_str()
                )));
            }
            None => return Ok(None),
        };
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
        let msg: &str = &data.message;

        if msg == "!blague" {
            let mut blagues = self.store.list(&data.team_id)?;
            let mut r = self.rng.gen_range(0, blagues.len() * 2);
            if r > blagues.len() - 1 {
                match self.rand_blague()? {
                    Some(t) => {
                        let _ = self.store.add(&data.team_id, &t);
                        let b = MBlague {
                            id: -1,
                            team_id: "".to_string(),
                            text: t,
                        };
                        blagues.push(b);
                        r = blagues.len() - 1;
                    }
                    None => {}
                };
            }
            match blagues.get(r) {
                Some(blague) => return Ok(client.send_message(data, &blague.text)?),
                None => return Ok(client.debug("ya une merdouille avec les blagues")?),
            };
        } else if msg == "!blague list" {
            let blagues = self.store.list(&data.team_id)?;
            let mut rep = String::from("Liste des blagounettes:\n");
            for blague in blagues {
                rep.push_str(&format!(" * {}: {}\n", blague.id, &blague.text));
            }

            return Ok(client.send_message(data, &rep)?);
        }

        match self.match_del.captures(msg) {
            Some(captures) => {
                match captures.get(1).unwrap().as_str().trim().parse() {
                    Ok(num) => {
                        self.store.del(&data.team_id, num)?;
                        return Ok(client.send_reaction(data, "ok_hand")?);
                    }
                    Err(e) => return Ok(client.send_reply(data, &format!("beurk: {:?}", e))?),
                };
            }
            None => {}
        };

        if msg.starts_with("!blague ") {
            match msg.splitn(2, " ").collect::<Vec<&str>>().get(1) {
                Some(blague) => {
                    if blague.len() > 300 {
                        return Ok(client
                            .send_reply(data, "la blague est trop longue. max 300 caractères")?);
                    }
                    self.store.add(&data.team_id, blague)?;
                    return Ok(client.send_reaction(data, "ok_hand")?);
                }
                None => {
                    return Ok(client.send_reply(data, "t’as des gros doigts papa")?);
                }
            }
        }

        Ok(())
    }
}
