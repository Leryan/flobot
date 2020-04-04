use crate::client::Client;
use crate::models::db;
use crate::models::GenericPost;

use regex::Regex;

pub trait Handler<C> {
    type Data;
    fn handle(&self, data: Self::Data, client: &C);
}

pub struct Debug {
    name: String,
}

impl Debug {
    pub fn new(name: &str) -> Self {
        Debug {
            name: String::from(name),
        }
    }
}

impl<C: Client> Handler<C> for Debug {
    type Data = GenericPost;

    fn handle(&self, data: GenericPost, _client: &C) {
        println!("handler {:?} -> {:?}", self.name, data)
    }
}

pub struct Trigger {
    dbpool: diesel::SqliteConnection,
    match_list: Regex,
    match_del: Regex,
    match_text: Regex,
    match_reaction: Regex,
}

impl Trigger {
    pub fn new(pool: diesel::SqliteConnection) -> Self {
        Self {
            dbpool: pool,
            match_list: Regex::new("^!trigger list.*$").unwrap(),
            match_del: Regex::new("^!trigger del \"(.+)\".*").unwrap(),
            match_reaction: Regex::new("^!trigger reaction \"([^\"]+)\" [:\"]([^:]+)[:\"].*$")
                .unwrap(),
            match_text: Regex::new("^!trigger text \"([^\"]+)\" \"([^\"]+)\".*$").unwrap(),
        }
    }
}

impl<C: Client> Handler<C> for Trigger {
    type Data = GenericPost;

    fn handle(&self, data: GenericPost, client: &C) {
        use crate::diesel::prelude::*;
        use crate::schema::trigger::dsl::*;
        let message = data.message.as_str();

        if !message.starts_with("!trigger ") {
            let res = trigger
                .filter(team_id.eq(data.team_id.as_str()))
                .load::<db::Trigger>(&self.dbpool)
                .unwrap_or(vec![]);
            for t in res {
                let tb = t.triggered_by.as_str();
                let tb_word = format!(" {} ", tb);
                let tb_start = format!("{} ", tb);
                let tb_end = format!(" {}", tb);
                if message.contains(tb_word.as_str())
                    || message.starts_with(tb_start.as_str())
                    || message.ends_with(tb_end.as_str())
                    || message == t.triggered_by.as_str()
                {
                    if t.text_.is_some() {
                        client.send_reply(data.clone(), t.text_.unwrap().as_str());
                        break;
                    } else {
                        client.send_reaction(data.clone(), t.emoji.unwrap().as_str());
                    }
                }
            }
            return;
        }

        if self.match_list.is_match(message) {
            let res = trigger
                .filter(team_id.eq(data.team_id.as_str()))
                .order(triggered_by.asc())
                .load::<db::Trigger>(&self.dbpool);
            match res {
                Ok(it) => {
                    client.send_trigger_list(it, data);
                }
                Err(e) => {
                    let mut post = data;
                    post.message = format!("tout cassé: {:?}", e).to_string();
                    client.send_post(post);
                }
            }
            return;
        }
        match self.match_text.captures(message) {
            Some(captures) => {
                use crate::schema::trigger;

                let new_trigger = db::NewTrigger {
                    triggered_by: captures.get(1).unwrap().as_str(),
                    emoji: None,
                    text_: Some(captures.get(2).unwrap().as_str()),
                    team_id: data.team_id.as_str(),
                };

                let _res = diesel::insert_into(trigger::table)
                    .values(&new_trigger)
                    .execute(&self.dbpool);

                client.send_reaction(data.clone(), "ok_hand");
                return;
            }
            None => {}
        }

        match self.match_reaction.captures(message) {
            Some(captures) => {
                use crate::schema::trigger;

                let new_trigger = db::NewTrigger {
                    triggered_by: captures.get(1).unwrap().as_str(),
                    emoji: Some(captures.get(2).unwrap().as_str()),
                    text_: None,
                    team_id: data.team_id.as_str(),
                };

                let _res = diesel::insert_into(trigger::table)
                    .values(&new_trigger)
                    .execute(&self.dbpool);

                client.send_reaction(data.clone(), "ok_hand");
                return;
            }
            None => {}
        }
        match self.match_del.captures(message) {
            Some(captures) => {
                let tb = captures.get(1).unwrap().as_str();
                let tid = data.team_id.as_str();
                let filter = triggered_by.eq(tb).and(team_id.eq(tid));
                let _res = diesel::delete(trigger.filter(filter)).execute(&self.dbpool);

                client.send_reaction(data.clone(), "ok_hand");
                return;
            }
            None => {}
        }
    }
}
