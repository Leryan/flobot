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
    remotes: R,
    client: Rc<C>,
}

impl<R, S, C> Blague<R, S, C> {
    pub fn new(store: Rc<S>, remotes: R, client: Rc<C>) -> Self {
        Blague {
            match_del: Regex::new(r"^!blague del (.*)").expect("cannot compile blague match del regex"),
            store,
            remotes,
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

    fn handle(&self, post: &GenericPost) -> Result {
        let msg = post.message.as_str();

        if msg == "!blague" {
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
