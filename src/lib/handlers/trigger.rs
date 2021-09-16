use crate::client;
use crate::db;
use crate::db::tempo::Tempo;
use crate::handlers::{Handler, Result};
use crate::models::GenericPost;
use regex::Regex;
use std::rc::Rc;
use std::time::Duration;

pub struct Trigger<C, E> {
    db: Rc<E>,
    client: Rc<C>,
    match_list: Regex,
    match_del: Regex,
    match_text: Regex,
    match_reaction: Regex,
    tempo: Tempo<String>,
    delay_repeat: Duration,
}

impl<C, E> Trigger<C, E> {
    pub fn new(db: Rc<E>, client: Rc<C>, tempo: Tempo<String>, delay_repeat: Duration) -> Self {
        Self {
            db,
            client,
            tempo,
            delay_repeat,
            match_list: Regex::new("^!trigger list.*$").unwrap(),
            match_del: Regex::new("^!trigger del \"(.+)\".*").unwrap(),
            match_reaction: Regex::new("^!trigger reaction \"([^\"]+)\" [:\"]([^:]+)[:\"].*$").unwrap(),
            match_text: Regex::new("^!trigger text \"([^\"]+)\" \"([^\"]+)\".*$").unwrap(),
        }
    }
}

impl<C, E> Handler for Trigger<C, E>
where
    C: client::Sender,
    E: db::Trigger,
{
    type Data = GenericPost;

    fn name(&self) -> &str {
        "trigger"
    }
    fn help(&self) -> Option<&str> {
        Some(
            "```
!trigger list
!trigger text \"trigger\" \"me\"\
!trigger reaction \"trigger\" :emoji:
!trigger del \"trigger\"
```",
        )
    }

    fn handle(&self, post: &GenericPost) -> Result {
        let message = post.message.as_str();

        if !message.starts_with("!trigger ") {
            let tempo_rate = format!("{}{}--rate-limit", &post.team_id, &post.channel_id);
            if self.tempo.exists(tempo_rate.clone()) {
                return Ok(());
            } else {
                self.tempo.set(tempo_rate.clone(), Duration::from_secs(3));
            }
            let res = self.db.search(&post.team_id)?;
            for t in res {
                let tb = &t.triggered_by;
                let tb_word = &format!(" {} ", tb);
                let tb_start = &format!("{} ", tb);
                let tb_end = &format!(" {}", tb);
                if message.contains(tb_word) || message.starts_with(tb_start) || message.ends_with(tb_end) || message == t.triggered_by {
                    let tempo_key = format!("{}{}{}", &post.team_id, &post.channel_id, tb);

                    // sending this trigger has been delayed
                    if self.tempo.exists(tempo_key.clone()) {
                        self.tempo.set(tempo_key.clone(), self.delay_repeat);
                        continue;
                    }
                    self.tempo.set(tempo_key.clone(), self.delay_repeat);

                    if t.text_.is_some() {
                        self.client.reply(post, &t.text_.unwrap())?;
                        break; // text is sorted after emoji, so we can break here: emoji were already processed.
                    } else {
                        self.client.reaction(post, &t.emoji.unwrap())?;
                    }
                }
            }
            return Ok(());
        }

        if self.match_list.is_match(message) {
            let res = self.db.list(&post.team_id)?;
            return Ok(self.client.send_trigger_list(res, post)?);
        }

        match self.match_text.captures(message) {
            Some(captures) => {
                let _ = self
                    .db
                    .add_text(&post.team_id, captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str());
                return Ok(self.client.reaction(post, "ok_hand")?);
            }
            None => {}
        }

        match self.match_reaction.captures(message) {
            Some(captures) => {
                let _ = self
                    .db
                    .add_emoji(&post.team_id, captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str());
                return Ok(self.client.reaction(post, "ok_hand")?);
            }
            None => {}
        }
        match self.match_del.captures(message) {
            Some(captures) => {
                let _ = self.db.del(&post.team_id, captures.get(1).unwrap().as_str())?;
                return Ok(self.client.reaction(post, "ok_hand")?);
            }
            None => {}
        }

        Ok(())
    }
}
