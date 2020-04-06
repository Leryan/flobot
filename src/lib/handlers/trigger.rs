use crate::client::Client;
use crate::db;
use crate::db::tempo::Tempo;
use crate::handlers::{Handler, Result};
use crate::models::GenericPost;
use regex::Regex;
use std::rc::Rc;
use std::time::Duration;

pub struct Trigger<E> {
    db: Rc<E>,
    match_list: Regex,
    match_del: Regex,
    match_text: Regex,
    match_reaction: Regex,
    tempo: Tempo,
    delay_repeat: Duration,
}

impl<E: db::Trigger> Trigger<E> {
    pub fn new(db: Rc<E>, tempo: Tempo, delay_repeat: Duration) -> Self {
        Self {
            db,
            tempo,
            delay_repeat,
            match_list: Regex::new("^!trigger list.*$").unwrap(),
            match_del: Regex::new("^!trigger del \"(.+)\".*").unwrap(),
            match_reaction: Regex::new("^!trigger reaction \"([^\"]+)\" [:\"]([^:]+)[:\"].*$")
                .unwrap(),
            match_text: Regex::new("^!trigger text \"([^\"]+)\" \"([^\"]+)\".*$").unwrap(),
        }
    }
}

impl<C: Client, E: db::Trigger> Handler<C> for Trigger<E> {
    type Data = GenericPost;

    fn handle(&mut self, data: GenericPost, client: &C) -> Result {
        let message = data.message.as_str();

        if !message.starts_with("!trigger ") {
            let res = self.db.search(data.team_id.as_str())?;
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
                        let tempo_key = format!(
                            "{}{}{}",
                            data.team_id.as_str(),
                            data.channel_id.as_str(),
                            tb
                        );

                        // sending this trigger has been delayed
                        if self.tempo.exists(tempo_key.as_str()) {
                            self.tempo.set(tempo_key.as_str(), self.delay_repeat);
                            break;
                        }

                        // now, delay this trigger
                        self.tempo.set(tempo_key.as_str(), self.delay_repeat);
                        client.send_reply(data.clone(), t.text_.unwrap().as_str())?;
                        break;
                    } else {
                        client.send_reaction(data.clone(), t.emoji.unwrap().as_str())?;
                    }
                }
            }
            return Ok(());
        }

        if self.match_list.is_match(message) {
            let res = self.db.list(data.team_id.as_str())?;
            return Ok(client.send_trigger_list(res, data)?);
        }

        match self.match_text.captures(message) {
            Some(captures) => {
                let _ = self.db.add_text(
                    data.team_id.as_str(),
                    captures.get(1).unwrap().as_str(),
                    captures.get(2).unwrap().as_str(),
                );
                return Ok(client.send_reaction(data.clone(), "ok_hand")?);
            }
            None => {}
        }

        match self.match_reaction.captures(message) {
            Some(captures) => {
                let _ = self.db.add_emoji(
                    data.team_id.as_str(),
                    captures.get(1).unwrap().as_str(),
                    captures.get(2).unwrap().as_str(),
                );
                return Ok(client.send_reaction(data.clone(), "ok_hand")?);
            }
            None => {}
        }
        match self.match_del.captures(message) {
            Some(captures) => {
                let _ = self
                    .db
                    .del(data.team_id.as_str(), captures.get(1).unwrap().as_str())?;
                return Ok(client.send_reaction(data.clone(), "ok_hand")?);
            }
            None => {}
        }

        Ok(())
    }
}
